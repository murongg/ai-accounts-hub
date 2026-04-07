use std::sync::Mutex;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use tauri::{AppHandle, Emitter};

use crate::codex_accounts::paths::CodexAccountPaths;

use super::models::CodexRefreshSettings;
use super::service::CodexUsageService;
use super::store::load_refresh_settings;

enum SchedulerCommand {
    RefreshNow {
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

    pub async fn refresh_now(&self) -> Result<(), String> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| "scheduler lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "scheduler not initialized".to_string())?;
        let (tx, rx) = oneshot::channel();
        sender
            .send(SchedulerCommand::RefreshNow { respond_to: tx })
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
        let _ = run_refresh_cycle(app.clone(), paths.clone()).await;
    }

    loop {
        if settings.enabled {
            let delay = tokio::time::sleep(Duration::from_secs(settings.interval_seconds));
            tokio::pin!(delay);

            tokio::select! {
                _ = &mut delay => {
                    let _ = run_refresh_cycle(app.clone(), paths.clone()).await;
                }
                command = receiver.recv() => {
                    match command {
                        Some(SchedulerCommand::RefreshNow { respond_to }) => {
                            let _ = respond_to.send(run_refresh_cycle(app.clone(), paths.clone()).await);
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
                Some(SchedulerCommand::RefreshNow { respond_to }) => {
                    let _ = respond_to.send(run_refresh_cycle(app.clone(), paths.clone()).await);
                }
                Some(SchedulerCommand::UpdateSettings(next)) => {
                    settings = next;
                    if settings.enabled {
                        let _ = run_refresh_cycle(app.clone(), paths.clone()).await;
                    }
                }
                None => break,
            }
        }
    }
}

async fn run_refresh_cycle(app: AppHandle, paths: CodexAccountPaths) -> Result<(), String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        CodexUsageService::with_process_fetcher(paths).refresh_all()
    })
    .await
    .map_err(|error| error.to_string())?;

    result?;
    app.emit("codex-usage-updated", ())
        .map_err(|error| error.to_string())?;
    Ok(())
}
