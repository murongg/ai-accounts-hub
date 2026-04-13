use std::sync::Mutex;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use tauri::{AppHandle, Emitter};

use crate::account_auto_switch::{
    select_claude_auto_switch_target, select_codex_auto_switch_target,
    select_gemini_auto_switch_target,
};
use crate::app_settings::store::load_app_settings;
use crate::claude_accounts::{paths::ClaudeAccountPaths, service::ClaudeAccountService};
use crate::claude_usage::service::ClaudeUsageService;
use crate::codex_accounts::{paths::CodexAccountPaths, service::CodexAccountService};
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
    UpdateSettings(CodexRefreshSettings),
}

#[derive(Default)]
pub struct CodexUsageSchedulerState {
    sender: Mutex<Option<mpsc::UnboundedSender<SchedulerCommand>>>,
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

        tauri::async_runtime::spawn(run_scheduler_loop(app, paths, settings, rx));
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

    pub async fn refresh_gemini_now(&self) -> Result<(), String> {
        self.refresh_target(RefreshTarget::Gemini).await
    }

    pub async fn refresh_claude_now(&self) -> Result<(), String> {
        self.refresh_target(RefreshTarget::Claude).await
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
) {
    if settings.enabled {
        let _ = run_refresh_cycle(app.clone(), paths.clone(), RefreshTarget::All).await;
    }

    loop {
        if settings.enabled {
            let delay = tokio::time::sleep(Duration::from_secs(settings.interval_seconds));
            tokio::pin!(delay);

            tokio::select! {
                _ = &mut delay => {
                    let _ = run_refresh_cycle(app.clone(), paths.clone(), RefreshTarget::All).await;
                }
                command = receiver.recv() => {
                    match command {
                        Some(SchedulerCommand::Refresh { target, respond_to }) => {
                            let _ = respond_to.send(run_refresh_cycle(app.clone(), paths.clone(), target).await);
                        }
                        Some(SchedulerCommand::UpdateSettings(next)) => {
                            settings = next;
                        }
                        None => break,
                    }
                }
            }
        } else {
            match receiver.recv().await {
                Some(SchedulerCommand::Refresh { target, respond_to }) => {
                    let _ = respond_to
                        .send(run_refresh_cycle(app.clone(), paths.clone(), target).await);
                }
                Some(SchedulerCommand::UpdateSettings(next)) => {
                    settings = next;
                    if settings.enabled {
                        let _ =
                            run_refresh_cycle(app.clone(), paths.clone(), RefreshTarget::All).await;
                    }
                }
                None => break,
            }
        }
    }
}

async fn run_refresh_cycle(
    app: AppHandle,
    paths: CodexAccountPaths,
    target: RefreshTarget,
) -> Result<(), String> {
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
            let auto_switch_enabled = load_auto_switch_enabled(&paths);

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
                    auto_switch_when_enabled(auto_switch_enabled, || {
                        auto_switch_codex_account_if_needed(&codex_switch_paths)
                    })
                },
                || {
                    auto_switch_when_enabled(auto_switch_enabled, || {
                        auto_switch_claude_account_if_needed(&claude_switch_paths)
                    })
                },
                || {
                    auto_switch_when_enabled(auto_switch_enabled, || {
                        auto_switch_gemini_account_if_needed(&gemini_switch_paths)
                    })
                },
            ))
        })
        .await
        .map_err(|error| error.to_string())??;

    emit_refresh_events(&app, &outcome)?;
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

fn auto_switch_when_enabled<AutoSwitch>(
    enabled: bool,
    auto_switch: AutoSwitch,
) -> Result<Option<String>, String>
where
    AutoSwitch: FnOnce() -> Result<Option<String>, String>,
{
    if !enabled {
        return Ok(None);
    }

    auto_switch()
}

fn load_auto_switch_enabled(paths: &CodexAccountPaths) -> bool {
    load_app_settings(paths)
        .map(|settings| settings.auto_switch_enabled)
        .unwrap_or(false)
}

fn auto_switch_codex_account_if_needed(
    paths: &CodexAccountPaths,
) -> Result<Option<String>, String> {
    let service = CodexAccountService::with_process_runner(paths.clone());
    let accounts = service.list_accounts()?;
    let Some(account_id) = select_codex_auto_switch_target(&accounts) else {
        return Ok(None);
    };

    service.switch_account(&account_id)?;
    Ok(Some(account_id))
}

fn auto_switch_claude_account_if_needed(
    paths: &ClaudeAccountPaths,
) -> Result<Option<String>, String> {
    let mut service = ClaudeAccountService::with_process_runner(paths.clone());
    let accounts = service.list_accounts()?;
    let Some(account_id) = select_claude_auto_switch_target(&accounts) else {
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
    use super::{auto_switch_when_enabled, run_refresh_actions, RefreshTarget};

    fn no_auto_switch() -> impl FnMut() -> Result<Option<String>, String> {
        || Ok(None)
    }

    #[test]
    fn auto_switch_guard_skips_action_when_setting_is_disabled() {
        let mut calls = 0;

        let result = auto_switch_when_enabled(false, || {
            calls += 1;
            Ok(Some("candidate".to_string()))
        });

        assert_eq!(result, Ok(None));
        assert_eq!(calls, 0);
    }

    #[test]
    fn auto_switch_guard_runs_action_when_setting_is_enabled() {
        let mut calls = 0;

        let result = auto_switch_when_enabled(true, || {
            calls += 1;
            Ok(Some("candidate".to_string()))
        });

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
}
