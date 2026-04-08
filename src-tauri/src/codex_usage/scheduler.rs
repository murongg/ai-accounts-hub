use std::sync::Mutex;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use tauri::{AppHandle, Emitter};

use crate::codex_accounts::paths::CodexAccountPaths;
use crate::gemini_accounts::paths::GeminiAccountPaths;
use crate::gemini_usage::service::GeminiUsageService;

use super::models::CodexRefreshSettings;
use super::service::CodexUsageService;
use super::store::load_refresh_settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshTarget {
    Codex,
    Gemini,
    All,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RefreshOutcome {
    successful_targets: Vec<RefreshTarget>,
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
        let mut sender = self.sender.lock().map_err(|_| "scheduler lock poisoned".to_string())?;
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

    async fn refresh_target(&self, target: RefreshTarget) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| "scheduler lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "scheduler not initialized".to_string())?;
        let (tx, rx) = oneshot::channel();
        sender
            .send(SchedulerCommand::Refresh { target, respond_to: tx })
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
                    let _ = respond_to.send(run_refresh_cycle(app.clone(), paths.clone(), target).await);
                }
                Some(SchedulerCommand::UpdateSettings(next)) => {
                    settings = next;
                    if settings.enabled {
                        let _ = run_refresh_cycle(app.clone(), paths.clone(), RefreshTarget::All).await;
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

    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let gemini_paths = GeminiAccountPaths::from_roots(paths.app_data_dir.clone(), home_dir);

        run_refresh_actions(
            target,
            || CodexUsageService::with_process_fetcher(paths.clone()).refresh_all(),
            || GeminiUsageService::with_process_fetcher(gemini_paths.clone()).refresh_all(),
        )
    })
    .await
    .map_err(|error| error.to_string())?;

    emit_refresh_events(&app, &outcome)?;
    outcome.error_message().map_or(Ok(()), Err)
}

fn run_refresh_actions<F1, F2>(
    target: RefreshTarget,
    mut refresh_codex: F1,
    mut refresh_gemini: F2,
) -> RefreshOutcome
where
    F1: FnMut() -> Result<(), String>,
    F2: FnMut() -> Result<(), String>,
{
    let mut outcome = RefreshOutcome::default();

    if matches!(target, RefreshTarget::Codex | RefreshTarget::All) {
        match refresh_codex() {
            Ok(()) => outcome.successful_targets.push(RefreshTarget::Codex),
            Err(error) => outcome
                .errors
                .push(format!("Codex refresh failed: {error}")),
        }
    }

    if matches!(target, RefreshTarget::Gemini | RefreshTarget::All) {
        match refresh_gemini() {
            Ok(()) => outcome.successful_targets.push(RefreshTarget::Gemini),
            Err(error) => outcome
                .errors
                .push(format!("Gemini refresh failed: {error}")),
        }
    }

    outcome
}

fn emit_refresh_events(app: &AppHandle, outcome: &RefreshOutcome) -> Result<(), String> {
    for target in &outcome.successful_targets {
        match target {
            RefreshTarget::Codex => app
                .emit("codex-usage-updated", ())
                .map_err(|error| error.to_string())?,
            RefreshTarget::Gemini => app
                .emit("gemini-usage-updated", ())
                .map_err(|error| error.to_string())?,
            RefreshTarget::All => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{run_refresh_actions, RefreshTarget};

    #[test]
    fn gemini_only_refresh_does_not_call_codex_refresh() {
        let mut codex_calls = 0;
        let mut gemini_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::Gemini,
            || {
                codex_calls += 1;
                Ok(())
            },
            || {
                gemini_calls += 1;
                Ok(())
            },
        );

        assert_eq!(codex_calls, 0);
        assert_eq!(gemini_calls, 1);
        assert_eq!(outcome.successful_targets, vec![RefreshTarget::Gemini]);
        assert_eq!(outcome.error_message(), None);
    }

    #[test]
    fn refresh_all_continues_to_gemini_when_codex_refresh_fails() {
        let mut codex_calls = 0;
        let mut gemini_calls = 0;

        let outcome = run_refresh_actions(
            RefreshTarget::All,
            || {
                codex_calls += 1;
                Err("codex unavailable".to_string())
            },
            || {
                gemini_calls += 1;
                Ok(())
            },
        );

        assert_eq!(codex_calls, 1);
        assert_eq!(gemini_calls, 1);
        assert_eq!(outcome.successful_targets, vec![RefreshTarget::Gemini]);
        assert_eq!(
            outcome.error_message(),
            Some("Codex refresh failed: codex unavailable".to_string())
        );
    }
}
