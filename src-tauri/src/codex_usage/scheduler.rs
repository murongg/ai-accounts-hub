use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::{mpsc, oneshot};

use tauri::{AppHandle, Emitter};

use crate::account_auto_switch::{
    select_claude_auto_switch_target_with_thresholds,
    select_codex_auto_switch_target_with_thresholds,
    select_codex_switch_candidate_above_thresholds, select_gemini_auto_switch_target,
    AutoSwitchThresholds,
};
use crate::app_settings::store::load_app_settings;
use crate::claude_accounts::{paths::ClaudeAccountPaths, service::ClaudeAccountService};
use crate::claude_usage::service::ClaudeUsageService;
use crate::codex_accounts::{
    models::CodexAccountListItem, paths::CodexAccountPaths, service::CodexAccountService,
};
use crate::gemini_accounts::{paths::GeminiAccountPaths, service::GeminiAccountService};
use crate::gemini_usage::service::GeminiUsageService;

use super::models::CodexRefreshSettings;
use super::service::CodexUsageService;
use super::store::load_refresh_settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshTarget {
    Codex,
    Claude,
    Gemini,
    All,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RefreshOutcome {
    successful_targets: Vec<RefreshTarget>,
    auto_switched_targets: Vec<RefreshTarget>,
    errors: Vec<String>,
}

impl RefreshOutcome {
    fn error_message(&self) -> Option<String> {
        if self.errors.is_empty() {
            None
        } else {
            Some(self.errors.join("; "))
        }
    }
}

enum SchedulerCommand {
    Refresh {
        target: RefreshTarget,
        respond_to: oneshot::Sender<Result<(), String>>,
    },
    RefreshCodexAccount {
        account_id: String,
        respond_to: oneshot::Sender<Result<(), String>>,
    },
    UpdateSettings(CodexRefreshSettings),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AutoSwitchSettings {
    enabled: bool,
    five_hour_threshold_percent: u8,
    weekly_threshold_percent: u8,
}

const ACCELERATED_CODEX_REFRESH_LIMIT: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceleratedCodexRefreshEntry {
    refresh_at: String,
    attempt_count: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AcceleratedCodexRefreshState {
    by_account_id: HashMap<String, AcceleratedCodexRefreshEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AcceleratedCodexRefreshAction {
    Continue { next_attempt_count: u8 },
    StopAndForceSwitch,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DueAcceleratedCodexRefresh {
    account_id: String,
    attempt_count: u8,
}

pub struct CodexUsageSchedulerState {
    sender: Mutex<Option<mpsc::UnboundedSender<SchedulerCommand>>>,
    accelerated_codex_refresh_state: Arc<Mutex<AcceleratedCodexRefreshState>>,
}

impl Default for CodexUsageSchedulerState {
    fn default() -> Self {
        Self {
            sender: Mutex::new(None),
            accelerated_codex_refresh_state: Arc::new(Mutex::new(
                AcceleratedCodexRefreshState::default(),
            )),
        }
    }
}

impl CodexUsageSchedulerState {
    pub fn initialize(&self, app: AppHandle, paths: CodexAccountPaths) -> Result<(), String> {
        let mut sender = self
            .sender
            .lock()
            .map_err(|_| "scheduler lock poisoned".to_string())?;
        if sender.is_some() {
            return Ok(());
        }

        let settings = load_refresh_settings(&paths)?;
        let (tx, rx) = mpsc::unbounded_channel();
        *sender = Some(tx);

        tauri::async_runtime::spawn(run_scheduler_loop(
            app,
            paths,
            settings,
            rx,
            Arc::clone(&self.accelerated_codex_refresh_state),
        ));
        Ok(())
    }

    pub fn update_settings(&self, settings: CodexRefreshSettings) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| "scheduler lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "scheduler not initialized".to_string())?;

        sender
            .send(SchedulerCommand::UpdateSettings(settings))
            .map_err(|_| "scheduler task is no longer running".to_string())
    }

    pub async fn refresh_all_now(&self) -> Result<(), String> {
        self.refresh_target(RefreshTarget::All).await
    }

    pub async fn refresh_codex_now(&self) -> Result<(), String> {
        self.refresh_target(RefreshTarget::Codex).await
    }

    pub async fn refresh_codex_account_now(&self, account_id: String) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| "scheduler lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "scheduler not initialized".to_string())?;
        let (tx, rx) = oneshot::channel();
        sender
            .send(SchedulerCommand::RefreshCodexAccount {
                account_id,
                respond_to: tx,
            })
            .map_err(|_| "scheduler task is no longer running".to_string())?;
        rx.await
            .map_err(|_| "scheduler response channel closed".to_string())?
    }

    pub async fn refresh_gemini_now(&self) -> Result<(), String> {
        self.refresh_target(RefreshTarget::Gemini).await
    }

    pub async fn refresh_claude_now(&self) -> Result<(), String> {
        self.refresh_target(RefreshTarget::Claude).await
    }

    pub fn apply_accelerated_refresh_state(
        &self,
        accounts: &mut [CodexAccountListItem],
    ) -> Result<(), String> {
        let state = self
            .accelerated_codex_refresh_state
            .lock()
            .map_err(|_| "accelerated refresh state lock poisoned".to_string())?;

        for account in accounts {
            account.refresh_accelerated_until = state
                .by_account_id
                .get(&account.id)
                .map(|entry| entry.refresh_at.clone());
        }

        Ok(())
    }

    async fn refresh_target(&self, target: RefreshTarget) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| "scheduler lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "scheduler not initialized".to_string())?;
        let (tx, rx) = oneshot::channel();
        sender
            .send(SchedulerCommand::Refresh {
                target,
                respond_to: tx,
            })
            .map_err(|_| "scheduler task is no longer running".to_string())?;
        rx.await
            .map_err(|_| "scheduler response channel closed".to_string())?
    }
}

async fn run_scheduler_loop(
    app: AppHandle,
    paths: CodexAccountPaths,
    mut settings: CodexRefreshSettings,
    mut receiver: mpsc::UnboundedReceiver<SchedulerCommand>,
    accelerated_codex_refresh_state: Arc<Mutex<AcceleratedCodexRefreshState>>,
) {
    let mut next_full_refresh_at = None;

    if settings.enabled {
        let _ = run_refresh_cycle(
            app.clone(),
            paths.clone(),
            RefreshTarget::All,
            Arc::clone(&accelerated_codex_refresh_state),
            true,
        )
        .await;
        next_full_refresh_at = Some(unix_timestamp_now().saturating_add(settings.interval_seconds));
    } else {
        clear_accelerated_codex_refresh_state(&accelerated_codex_refresh_state);
    }

    loop {
        if settings.enabled {
            let delay_seconds =
                next_scheduler_delay_seconds(next_full_refresh_at, &accelerated_codex_refresh_state);
            let delay = tokio::time::sleep(Duration::from_secs(delay_seconds));
            tokio::pin!(delay);

            tokio::select! {
                _ = &mut delay => {
                    let now = unix_timestamp_now();
                    if next_full_refresh_at.is_some_and(|deadline| deadline <= now) {
                        let _ = run_refresh_cycle(
                            app.clone(),
                            paths.clone(),
                            RefreshTarget::All,
                            Arc::clone(&accelerated_codex_refresh_state),
                            true,
                        ).await;
                        next_full_refresh_at = Some(unix_timestamp_now().saturating_add(settings.interval_seconds));
                    } else {
                        for refresh in due_accelerated_codex_account_entries(
                            &accelerated_codex_refresh_state,
                            now,
                        ) {
                            let _ = run_refresh_codex_account(
                                app.clone(),
                                paths.clone(),
                                refresh.account_id,
                                Arc::clone(&accelerated_codex_refresh_state),
                                true,
                                Some(refresh.attempt_count),
                            ).await;
                        }
                    }
                }
                command = receiver.recv() => {
                    match command {
                        Some(SchedulerCommand::Refresh { target, respond_to }) => {
                            let result = run_refresh_cycle(
                                app.clone(),
                                paths.clone(),
                                target,
                                Arc::clone(&accelerated_codex_refresh_state),
                                true,
                            ).await;
                            if matches!(target, RefreshTarget::Codex | RefreshTarget::All) {
                                next_full_refresh_at = Some(unix_timestamp_now().saturating_add(settings.interval_seconds));
                            }
                            let _ = respond_to.send(result);
                        }
                        Some(SchedulerCommand::RefreshCodexAccount { account_id, respond_to }) => {
                            let _ = respond_to.send(
                                run_refresh_codex_account(
                                    app.clone(),
                                    paths.clone(),
                                    account_id,
                                    Arc::clone(&accelerated_codex_refresh_state),
                                    true,
                                    None,
                                ).await
                            );
                        }
                        Some(SchedulerCommand::UpdateSettings(next)) => {
                            settings = next;
                            if settings.enabled {
                                next_full_refresh_at =
                                    Some(unix_timestamp_now().saturating_add(settings.interval_seconds));
                            } else {
                                clear_accelerated_codex_refresh_state(&accelerated_codex_refresh_state);
                                next_full_refresh_at = None;
                            }
                        }
                        None => break,
                    }
                }
            }
        } else {
            match receiver.recv().await {
                Some(SchedulerCommand::Refresh { target, respond_to }) => {
                    let _ = respond_to.send(
                        run_refresh_cycle(
                            app.clone(),
                            paths.clone(),
                            target,
                            Arc::clone(&accelerated_codex_refresh_state),
                            false,
                        ).await
                    );
                }
                Some(SchedulerCommand::RefreshCodexAccount { account_id, respond_to }) => {
                    let _ = respond_to.send(
                        run_refresh_codex_account(
                            app.clone(),
                            paths.clone(),
                            account_id,
                            Arc::clone(&accelerated_codex_refresh_state),
                            false,
                            None,
                        ).await
                    );
                }
                Some(SchedulerCommand::UpdateSettings(next)) => {
                    settings = next;
                    if settings.enabled {
                        let _ = run_refresh_cycle(
                            app.clone(),
                            paths.clone(),
                            RefreshTarget::All,
                            Arc::clone(&accelerated_codex_refresh_state),
                            true,
                        ).await;
                        next_full_refresh_at = Some(unix_timestamp_now().saturating_add(settings.interval_seconds));
                    } else {
                        clear_accelerated_codex_refresh_state(&accelerated_codex_refresh_state);
                        next_full_refresh_at = None;
                    }
                }
                None => break,
            }
        }
    }
}

async fn run_refresh_codex_account(
    app: AppHandle,
    paths: CodexAccountPaths,
    account_id: String,
    accelerated_codex_refresh_state: Arc<Mutex<AcceleratedCodexRefreshState>>,
    scheduler_enabled: bool,
    accelerated_attempt_count: Option<u8>,
) -> Result<(), String> {
    let refresh_account_id = account_id.clone();
    let scheduler_paths = paths.clone();
    let mut outcome =
        tauri::async_runtime::spawn_blocking(move || -> Result<RefreshOutcome, String> {
        let refresh_paths = scheduler_paths.clone();
        let switch_paths = scheduler_paths.clone();
        let auto_switch_settings = load_auto_switch_settings(&scheduler_paths);

        Ok(run_single_codex_refresh_action(
            || {
                CodexUsageService::with_process_fetcher(refresh_paths.clone())
                    .refresh_account(&refresh_account_id)
            },
            || {
                auto_switch_when_enabled(auto_switch_settings, |settings| {
                    auto_switch_codex_account_if_needed(&switch_paths, settings)
                })
            },
        ))
    })
    .await
    .map_err(|error| error.to_string())??;

    if scheduler_enabled {
        let auto_switch_settings = load_auto_switch_settings(&paths);
        let codex_auto_switched = outcome.auto_switched_targets.contains(&RefreshTarget::Codex);

        match accelerated_attempt_count
            .map(|attempt_count| {
                accelerated_codex_refresh_action_after_attempt(
                    attempt_count,
                    codex_auto_switched,
                )
            })
            .unwrap_or(AcceleratedCodexRefreshAction::Reset)
        {
            AcceleratedCodexRefreshAction::Continue { next_attempt_count } => {
                rebuild_accelerated_codex_refresh_state(
                    &paths,
                    auto_switch_settings,
                    &accelerated_codex_refresh_state,
                    Some((&account_id, next_attempt_count)),
                    None,
                )?;
            }
            AcceleratedCodexRefreshAction::StopAndForceSwitch => {
                if let Some(switched_account_id) = tauri::async_runtime::spawn_blocking({
                    let force_switch_paths = paths.clone();
                    move || {
                        force_switch_codex_account_above_thresholds(
                            &force_switch_paths,
                            auto_switch_settings,
                        )
                    }
                })
                .await
                .map_err(|error| error.to_string())??
                {
                    outcome.auto_switched_targets.push(RefreshTarget::Codex);
                    rebuild_accelerated_codex_refresh_state(
                        &paths,
                        auto_switch_settings,
                        &accelerated_codex_refresh_state,
                        None,
                        None,
                    )?;
                    let _ = switched_account_id;
                } else {
                    rebuild_accelerated_codex_refresh_state(
                        &paths,
                        auto_switch_settings,
                        &accelerated_codex_refresh_state,
                        None,
                        Some(&account_id),
                    )?;
                }
            }
            AcceleratedCodexRefreshAction::Reset => {
                rebuild_accelerated_codex_refresh_state(
                    &paths,
                    auto_switch_settings,
                    &accelerated_codex_refresh_state,
                    None,
                    None,
                )?;
            }
        }
    } else {
        clear_accelerated_codex_refresh_state(&accelerated_codex_refresh_state);
    }

    emit_refresh_events(&app, &outcome)?;
    outcome.error_message().map_or(Ok(()), Err)
}

async fn run_refresh_cycle(
    app: AppHandle,
    paths: CodexAccountPaths,
    target: RefreshTarget,
    accelerated_codex_refresh_state: Arc<Mutex<AcceleratedCodexRefreshState>>,
    scheduler_enabled: bool,
) -> Result<(), String> {
    let scheduler_paths = paths.clone();
    let home_dir = paths
        .system_codex_dir
        .parent()
        .ok_or_else(|| "failed to resolve home dir from Codex paths".to_string())?
        .to_path_buf();

    let outcome =
        tauri::async_runtime::spawn_blocking(move || -> Result<RefreshOutcome, String> {
            let claude_paths =
                ClaudeAccountPaths::from_roots(paths.app_data_dir.clone(), home_dir.clone());
            let gemini_paths = GeminiAccountPaths::from_roots(paths.app_data_dir.clone(), home_dir);

            let codex_refresh_paths = paths.clone();
            let codex_switch_paths = paths.clone();
            let claude_refresh_paths = claude_paths.clone();
            let claude_switch_paths = claude_paths.clone();
            let gemini_refresh_paths = gemini_paths.clone();
            let gemini_switch_paths = gemini_paths.clone();
            let auto_switch_settings = load_auto_switch_settings(&paths);

            Ok(run_refresh_actions(
                target,
                || {
                    CodexUsageService::with_process_fetcher(codex_refresh_paths.clone())
                        .refresh_all()
                },
                || {
                    ClaudeUsageService::with_process_fetchers(claude_refresh_paths.clone())
                        .refresh_all()
                },
                || {
                    GeminiUsageService::with_process_fetcher(gemini_refresh_paths.clone())
                        .refresh_all()
                },
                || {
                    auto_switch_when_enabled(auto_switch_settings, |settings| {
                        auto_switch_codex_account_if_needed(&codex_switch_paths, settings)
                    })
                },
                || {
                    auto_switch_when_enabled(auto_switch_settings, |settings| {
                        auto_switch_claude_account_if_needed(&claude_switch_paths, settings)
                    })
                },
                || {
                    auto_switch_when_enabled(auto_switch_settings, |_| {
                        auto_switch_gemini_account_if_needed(&gemini_switch_paths)
                    })
                },
            ))
        })
        .await
        .map_err(|error| error.to_string())??;

    emit_refresh_events(&app, &outcome)?;
    if matches!(target, RefreshTarget::Codex | RefreshTarget::All) {
        if scheduler_enabled {
            rebuild_accelerated_codex_refresh_state(
                &scheduler_paths,
                load_auto_switch_settings(&scheduler_paths),
                &accelerated_codex_refresh_state,
                None,
                None,
            )?;
        } else {
            clear_accelerated_codex_refresh_state(&accelerated_codex_refresh_state);
        }
    }
    outcome.error_message().map_or(Ok(()), Err)
}

fn run_refresh_actions<F1, F2, F3, S1, S2, S3>(
    target: RefreshTarget,
    mut refresh_codex: F1,
    mut refresh_claude: F2,
    mut refresh_gemini: F3,
    mut auto_switch_codex: S1,
    mut auto_switch_claude: S2,
    mut auto_switch_gemini: S3,
) -> RefreshOutcome
where
    F1: FnMut() -> Result<(), String>,
    F2: FnMut() -> Result<(), String>,
    F3: FnMut() -> Result<(), String>,
    S1: FnMut() -> Result<Option<String>, String>,
    S2: FnMut() -> Result<Option<String>, String>,
    S3: FnMut() -> Result<Option<String>, String>,
{
    let mut outcome = RefreshOutcome::default();

    if matches!(target, RefreshTarget::Codex | RefreshTarget::All) {
        run_provider_refresh_action(
            &mut outcome,
            RefreshTarget::Codex,
            "Codex",
            &mut refresh_codex,
            &mut auto_switch_codex,
        );
    }

    if matches!(target, RefreshTarget::Claude | RefreshTarget::All) {
        run_provider_refresh_action(
            &mut outcome,
            RefreshTarget::Claude,
            "Claude",
            &mut refresh_claude,
            &mut auto_switch_claude,
        );
    }

    if matches!(target, RefreshTarget::Gemini | RefreshTarget::All) {
        run_provider_refresh_action(
            &mut outcome,
            RefreshTarget::Gemini,
            "Gemini",
            &mut refresh_gemini,
            &mut auto_switch_gemini,
        );
    }

    outcome
}

fn run_provider_refresh_action<Refresh, AutoSwitch>(
    outcome: &mut RefreshOutcome,
    target: RefreshTarget,
    label: &str,
    refresh: &mut Refresh,
    auto_switch: &mut AutoSwitch,
) where
    Refresh: FnMut() -> Result<(), String>,
    AutoSwitch: FnMut() -> Result<Option<String>, String>,
{
    match refresh() {
        Ok(()) => {
            outcome.successful_targets.push(target);
            match auto_switch() {
                Ok(Some(_account_id)) => outcome.auto_switched_targets.push(target),
                Ok(None) => {}
                Err(error) => outcome
                    .errors
                    .push(format!("{label} auto switch failed: {error}")),
            }
        }
        Err(error) => outcome
            .errors
            .push(format!("{label} refresh failed: {error}")),
    }
}

fn run_single_codex_refresh_action<Refresh, AutoSwitch>(
    mut refresh_codex_account: Refresh,
    mut auto_switch_codex: AutoSwitch,
) -> RefreshOutcome
where
    Refresh: FnMut() -> Result<(), String>,
    AutoSwitch: FnMut() -> Result<Option<String>, String>,
{
    let mut outcome = RefreshOutcome::default();
    run_provider_refresh_action(
        &mut outcome,
        RefreshTarget::Codex,
        "Codex",
        &mut refresh_codex_account,
        &mut auto_switch_codex,
    );
    outcome
}

fn auto_switch_when_enabled<AutoSwitch>(
    settings: AutoSwitchSettings,
    auto_switch: AutoSwitch,
) -> Result<Option<String>, String>
where
    AutoSwitch: FnOnce(AutoSwitchSettings) -> Result<Option<String>, String>,
{
    if !settings.enabled {
        return Ok(None);
    }

    auto_switch(settings)
}

fn should_accelerate_codex_refresh(
    account: &CodexAccountListItem,
    settings: AutoSwitchSettings,
) -> bool {
    if !settings.enabled || !account.is_active || account.needs_relogin.unwrap_or(false) {
        return false;
    }

    let five_hour_warning = account.five_hour_remaining_percent.is_some_and(|value| {
        value <= settings.five_hour_threshold_percent.saturating_add(5)
    });
    let weekly_warning = account
        .weekly_remaining_percent
        .is_some_and(|value| value <= settings.weekly_threshold_percent.saturating_add(1));

    five_hour_warning || weekly_warning
}

fn accelerated_codex_refresh_action_after_attempt(
    previous_attempt_count: u8,
    auto_switched: bool,
) -> AcceleratedCodexRefreshAction {
    if auto_switched {
        return AcceleratedCodexRefreshAction::Reset;
    }

    let next_attempt_count = previous_attempt_count.saturating_add(1);
    if next_attempt_count >= ACCELERATED_CODEX_REFRESH_LIMIT {
        AcceleratedCodexRefreshAction::StopAndForceSwitch
    } else {
        AcceleratedCodexRefreshAction::Continue { next_attempt_count }
    }
}

fn rebuild_accelerated_codex_refresh_state(
    paths: &CodexAccountPaths,
    settings: AutoSwitchSettings,
    accelerated_codex_refresh_state: &Arc<Mutex<AcceleratedCodexRefreshState>>,
    carried_attempt: Option<(&str, u8)>,
    suppressed_account_id: Option<&str>,
) -> Result<(), String> {
    let service = CodexAccountService::with_process_runner(paths.clone());
    let next_refresh_at = timestamp_after_seconds(60);
    let by_account_id = service
        .list_accounts()?
        .into_iter()
        .filter(|account| should_accelerate_codex_refresh(account, settings))
        .filter(|account| suppressed_account_id != Some(account.id.as_str()))
        .map(|account| {
            let attempt_count = carried_attempt
                .filter(|(account_id, _)| *account_id == account.id.as_str())
                .map(|(_, attempt_count)| attempt_count)
                .unwrap_or(0);

            (
                account.id,
                AcceleratedCodexRefreshEntry {
                    refresh_at: next_refresh_at.clone(),
                    attempt_count,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let mut state = accelerated_codex_refresh_state
        .lock()
        .map_err(|_| "accelerated refresh state lock poisoned".to_string())?;
    state.by_account_id = by_account_id;
    Ok(())
}

fn clear_accelerated_codex_refresh_state(
    accelerated_codex_refresh_state: &Arc<Mutex<AcceleratedCodexRefreshState>>,
) {
    if let Ok(mut state) = accelerated_codex_refresh_state.lock() {
        state.by_account_id.clear();
    }
}

fn due_accelerated_codex_account_entries(
    accelerated_codex_refresh_state: &Arc<Mutex<AcceleratedCodexRefreshState>>,
    now: u64,
) -> Vec<DueAcceleratedCodexRefresh> {
    accelerated_codex_refresh_state
        .lock()
        .map(|state| {
            state
                .by_account_id
                .iter()
                .filter_map(|(account_id, entry)| {
                    parse_unix_timestamp(&entry.refresh_at)
                        .filter(|refresh_at| *refresh_at <= now)
                        .map(|_| DueAcceleratedCodexRefresh {
                            account_id: account_id.clone(),
                            attempt_count: entry.attempt_count,
                        })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn next_scheduler_delay_seconds(
    next_full_refresh_at: Option<u64>,
    accelerated_codex_refresh_state: &Arc<Mutex<AcceleratedCodexRefreshState>>,
) -> u64 {
    let now = unix_timestamp_now();
    let full_refresh_delay = next_full_refresh_at
        .map(|refresh_at| refresh_at.saturating_sub(now))
        .unwrap_or(u64::MAX);
    let accelerated_delay = accelerated_codex_refresh_state
        .lock()
        .map(|state| {
            state
                .by_account_id
                .values()
                .filter_map(|entry| parse_unix_timestamp(&entry.refresh_at))
                .map(|refresh_at| refresh_at.saturating_sub(now))
                .min()
                .unwrap_or(u64::MAX)
        })
        .unwrap_or(u64::MAX);

    full_refresh_delay.min(accelerated_delay).max(1)
}

fn timestamp_after_seconds(seconds: u64) -> String {
    unix_timestamp_now().saturating_add(seconds).to_string()
}

fn unix_timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn parse_unix_timestamp(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok()
}

fn load_auto_switch_settings(paths: &CodexAccountPaths) -> AutoSwitchSettings {
    load_app_settings(paths)
        .map(|settings| AutoSwitchSettings {
            enabled: settings.auto_switch_enabled,
            five_hour_threshold_percent: settings.auto_switch_five_hour_threshold_percent,
            weekly_threshold_percent: settings.auto_switch_weekly_threshold_percent,
        })
        .unwrap_or(AutoSwitchSettings {
            enabled: false,
            five_hour_threshold_percent: 0,
            weekly_threshold_percent: 0,
        })
}

fn auto_switch_codex_account_if_needed(
    paths: &CodexAccountPaths,
    settings: AutoSwitchSettings,
) -> Result<Option<String>, String> {
    let service = CodexAccountService::with_process_runner(paths.clone());
    let accounts = service.list_accounts()?;
    let Some(account_id) = select_codex_auto_switch_target_with_thresholds(
        &accounts,
        AutoSwitchThresholds {
            five_hour_percent: settings.five_hour_threshold_percent,
            weekly_percent: settings.weekly_threshold_percent,
        },
    ) else {
        return Ok(None);
    };

    service.switch_account(&account_id)?;
    Ok(Some(account_id))
}

fn force_switch_codex_account_above_thresholds(
    paths: &CodexAccountPaths,
    settings: AutoSwitchSettings,
) -> Result<Option<String>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    let service = CodexAccountService::with_process_runner(paths.clone());
    let accounts = service.list_accounts()?;
    let Some(account_id) = select_codex_switch_candidate_above_thresholds(
        &accounts,
        AutoSwitchThresholds {
            five_hour_percent: settings.five_hour_threshold_percent,
            weekly_percent: settings.weekly_threshold_percent,
        },
    ) else {
        return Ok(None);
    };

    service.switch_account(&account_id)?;
    Ok(Some(account_id))
}

fn auto_switch_claude_account_if_needed(
    paths: &ClaudeAccountPaths,
    settings: AutoSwitchSettings,
) -> Result<Option<String>, String> {
    let mut service = ClaudeAccountService::with_process_runner(paths.clone());
    let accounts = service.list_accounts()?;
    let Some(account_id) = select_claude_auto_switch_target_with_thresholds(
        &accounts,
        AutoSwitchThresholds {
            five_hour_percent: settings.five_hour_threshold_percent,
            weekly_percent: settings.weekly_threshold_percent,
        },
    ) else {
        return Ok(None);
    };

    service.switch_account(&account_id)?;
    Ok(Some(account_id))
}

fn auto_switch_gemini_account_if_needed(
    paths: &GeminiAccountPaths,
) -> Result<Option<String>, String> {
    let service = GeminiAccountService::with_process_runner(paths.clone());
    let accounts = service.list_accounts()?;
    let Some(account_id) = select_gemini_auto_switch_target(&accounts) else {
        return Ok(None);
    };

    service.switch_account(&account_id)?;
    Ok(Some(account_id))
}

fn emit_refresh_events(app: &AppHandle, outcome: &RefreshOutcome) -> Result<(), String> {
    for target in &outcome.successful_targets {
        match target {
            RefreshTarget::Codex => app
                .emit("codex-usage-updated", ())
                .map_err(|error| error.to_string())?,
            RefreshTarget::Claude => app
                .emit("claude-usage-updated", ())
                .map_err(|error| error.to_string())?,
            RefreshTarget::Gemini => app
                .emit("gemini-usage-updated", ())
                .map_err(|error| error.to_string())?,
            RefreshTarget::All => {}
        }
    }

    for target in &outcome.auto_switched_targets {
        emit_account_switched_event(app, *target)?;
    }

    crate::status_bar::refresh_status_menu(app)?;

    Ok(())
}

fn emit_account_switched_event(app: &AppHandle, target: RefreshTarget) -> Result<(), String> {
    let event_name = match target {
        RefreshTarget::Codex => "codex-account-switched",
        RefreshTarget::Claude => "claude-account-switched",
        RefreshTarget::Gemini => "gemini-account-switched",
        RefreshTarget::All => return Ok(()),
    };

    app.emit(event_name, ()).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        accelerated_codex_refresh_action_after_attempt, auto_switch_when_enabled,
        run_refresh_actions, AcceleratedCodexRefreshAction, AutoSwitchSettings, RefreshTarget,
    };
    use crate::codex_accounts::models::CodexAccountListItem;

    fn no_auto_switch() -> impl FnMut() -> Result<Option<String>, String> {
        || Ok(None)
    }

    fn codex_account(
        five_hour_remaining_percent: Option<u8>,
        weekly_remaining_percent: Option<u8>,
        needs_relogin: Option<bool>,
    ) -> CodexAccountListItem {
        CodexAccountListItem {
            id: "codex-test".to_string(),
            email: "codex@example.com".to_string(),
            label: None,
            plan: Some("Plus".to_string()),
            account_id: Some("acct-codex-test".to_string()),
            is_active: true,
            last_authenticated_at: "0".to_string(),
            five_hour_remaining_percent,
            weekly_remaining_percent,
            five_hour_refresh_at: None,
            weekly_refresh_at: None,
            last_synced_at: Some("1775900000".to_string()),
            last_sync_error: None,
            credits_balance: None,
            needs_relogin,
            refresh_accelerated_until: None,
        }
    }

    #[test]
    fn auto_switch_guard_skips_action_when_setting_is_disabled() {
        let mut calls = 0;

        let result = auto_switch_when_enabled(
            AutoSwitchSettings {
                enabled: false,
                five_hour_threshold_percent: 0,
                weekly_threshold_percent: 0,
            },
            |_| {
                calls += 1;
                Ok(Some("candidate".to_string()))
            },
        );

        assert_eq!(result, Ok(None));
        assert_eq!(calls, 0);
    }

    #[test]
    fn auto_switch_guard_runs_action_when_setting_is_enabled() {
        let mut calls = 0;

        let result = auto_switch_when_enabled(
            AutoSwitchSettings {
                enabled: true,
                five_hour_threshold_percent: 9,
                weekly_threshold_percent: 4,
            },
            |settings| {
                calls += 1;
                assert_eq!(settings.five_hour_threshold_percent, 9);
                assert_eq!(settings.weekly_threshold_percent, 4);
                Ok(Some("candidate".to_string()))
            },
        );

        assert_eq!(result, Ok(Some("candidate".to_string())));
        assert_eq!(calls, 1);
    }

    #[test]
    fn gemini_only_refresh_does_not_call_codex_refresh() {
        let mut codex_calls = 0;
        let mut claude_calls = 0;
        let mut gemini_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::Gemini,
            || {
                codex_calls += 1;
                Ok(())
            },
            || {
                claude_calls += 1;
                Ok(())
            },
            || {
                gemini_calls += 1;
                Ok(())
            },
            no_auto_switch(),
            no_auto_switch(),
            no_auto_switch(),
        );

        assert_eq!(codex_calls, 0);
        assert_eq!(claude_calls, 0);
        assert_eq!(gemini_calls, 1);
        assert_eq!(outcome.successful_targets, vec![RefreshTarget::Gemini]);
        assert_eq!(outcome.error_message(), None);
    }

    #[test]
    fn refresh_all_continues_to_other_targets_when_codex_refresh_fails() {
        let mut codex_calls = 0;
        let mut claude_calls = 0;
        let mut gemini_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::All,
            || {
                codex_calls += 1;
                Err("codex unavailable".to_string())
            },
            || {
                claude_calls += 1;
                Ok(())
            },
            || {
                gemini_calls += 1;
                Ok(())
            },
            no_auto_switch(),
            no_auto_switch(),
            no_auto_switch(),
        );

        assert_eq!(codex_calls, 1);
        assert_eq!(claude_calls, 1);
        assert_eq!(gemini_calls, 1);
        assert_eq!(
            outcome.successful_targets,
            vec![RefreshTarget::Claude, RefreshTarget::Gemini]
        );
        assert_eq!(
            outcome.error_message(),
            Some("Codex refresh failed: codex unavailable".to_string())
        );
    }

    #[test]
    fn claude_only_refresh_does_not_call_codex_or_gemini_refresh() {
        let mut codex_calls = 0;
        let mut claude_calls = 0;
        let mut gemini_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::Claude,
            || {
                codex_calls += 1;
                Ok(())
            },
            || {
                claude_calls += 1;
                Ok(())
            },
            || {
                gemini_calls += 1;
                Ok(())
            },
            no_auto_switch(),
            no_auto_switch(),
            no_auto_switch(),
        );

        assert_eq!(codex_calls, 0);
        assert_eq!(claude_calls, 1);
        assert_eq!(gemini_calls, 0);
        assert_eq!(outcome.successful_targets, vec![RefreshTarget::Claude]);
        assert_eq!(outcome.error_message(), None);
    }

    #[test]
    fn successful_refresh_records_targets_that_auto_switched_accounts() {
        let mut gemini_auto_switch_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::Gemini,
            || panic!("Codex should not refresh"),
            || panic!("Claude should not refresh"),
            || Ok(()),
            no_auto_switch(),
            no_auto_switch(),
            || {
                gemini_auto_switch_calls += 1;
                Ok(Some("gemini-high".to_string()))
            },
        );

        assert_eq!(gemini_auto_switch_calls, 1);
        assert_eq!(outcome.successful_targets, vec![RefreshTarget::Gemini]);
        assert_eq!(outcome.auto_switched_targets, vec![RefreshTarget::Gemini]);
        assert_eq!(outcome.error_message(), None);
    }

    #[test]
    fn failed_refresh_skips_auto_switch_for_that_target() {
        let mut codex_auto_switch_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::Codex,
            || Err("network unavailable".to_string()),
            || panic!("Claude should not refresh"),
            || panic!("Gemini should not refresh"),
            || {
                codex_auto_switch_calls += 1;
                Ok(Some("codex-high".to_string()))
            },
            no_auto_switch(),
            no_auto_switch(),
        );

        assert_eq!(codex_auto_switch_calls, 0);
        assert!(outcome.successful_targets.is_empty());
        assert!(outcome.auto_switched_targets.is_empty());
        assert_eq!(
            outcome.error_message(),
            Some("Codex refresh failed: network unavailable".to_string())
        );
    }

    #[test]
    fn single_codex_refresh_records_auto_switch_when_refresh_triggers_switch() {
        let mut auto_switch_calls = 0;

        let outcome = super::run_single_codex_refresh_action(
            || Ok(()),
            || {
                auto_switch_calls += 1;
                Ok(Some("codex-next".to_string()))
            },
        );

        assert_eq!(auto_switch_calls, 1);
        assert_eq!(outcome.successful_targets, vec![RefreshTarget::Codex]);
        assert_eq!(outcome.auto_switched_targets, vec![RefreshTarget::Codex]);
        assert_eq!(outcome.error_message(), None);
    }

    #[test]
    fn single_codex_refresh_skips_auto_switch_when_refresh_fails() {
        let mut auto_switch_calls = 0;

        let outcome = super::run_single_codex_refresh_action(
            || Err("network unavailable".to_string()),
            || {
                auto_switch_calls += 1;
                Ok(Some("codex-next".to_string()))
            },
        );

        assert_eq!(auto_switch_calls, 0);
        assert!(outcome.successful_targets.is_empty());
        assert!(outcome.auto_switched_targets.is_empty());
        assert_eq!(
            outcome.error_message(),
            Some("Codex refresh failed: network unavailable".to_string())
        );
    }

    #[test]
    fn codex_account_enters_accelerated_refresh_when_five_hour_reaches_threshold_plus_five() {
        let account = codex_account(Some(14), Some(88), Some(false));

        assert!(super::should_accelerate_codex_refresh(
            &account,
            AutoSwitchSettings {
                enabled: true,
                five_hour_threshold_percent: 9,
                weekly_threshold_percent: 20,
            }
        ));
    }

    #[test]
    fn codex_account_enters_accelerated_refresh_when_weekly_reaches_threshold_plus_one() {
        let account = codex_account(Some(80), Some(5), Some(false));

        assert!(super::should_accelerate_codex_refresh(
            &account,
            AutoSwitchSettings {
                enabled: true,
                five_hour_threshold_percent: 10,
                weekly_threshold_percent: 4,
            }
        ));
    }

    #[test]
    fn codex_account_leaves_accelerated_refresh_when_both_windows_leave_warning_zone() {
        let account = codex_account(Some(16), Some(8), Some(false));

        assert!(!super::should_accelerate_codex_refresh(
            &account,
            AutoSwitchSettings {
                enabled: true,
                five_hour_threshold_percent: 10,
                weekly_threshold_percent: 6,
            }
        ));
    }

    #[test]
    fn inactive_codex_account_does_not_enter_accelerated_refresh_even_when_warning_threshold_is_reached(
    ) {
        let mut account = codex_account(Some(14), Some(5), Some(false));
        account.is_active = false;

        assert!(!super::should_accelerate_codex_refresh(
            &account,
            AutoSwitchSettings {
                enabled: true,
                five_hour_threshold_percent: 9,
                weekly_threshold_percent: 4,
            }
        ));
    }

    #[test]
    fn accelerated_codex_refresh_continues_before_third_attempt_when_no_switch_happens() {
        assert_eq!(
            accelerated_codex_refresh_action_after_attempt(1, false),
            AcceleratedCodexRefreshAction::Continue { next_attempt_count: 2 }
        );
    }

    #[test]
    fn accelerated_codex_refresh_stops_and_forces_switch_on_third_attempt_without_switch() {
        assert_eq!(
            accelerated_codex_refresh_action_after_attempt(2, false),
            AcceleratedCodexRefreshAction::StopAndForceSwitch
        );
    }

    #[test]
    fn accelerated_codex_refresh_resets_when_refresh_already_switched_account() {
        assert_eq!(
            accelerated_codex_refresh_action_after_attempt(2, true),
            AcceleratedCodexRefreshAction::Reset
        );
    }

    #[test]
    fn accelerated_codex_refresh_still_stops_and_forces_switch_after_more_than_two_attempts() {
        assert_eq!(
            accelerated_codex_refresh_action_after_attempt(7, false),
            AcceleratedCodexRefreshAction::StopAndForceSwitch
        );
    }
}
