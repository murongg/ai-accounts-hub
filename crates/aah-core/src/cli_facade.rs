use serde::Serialize;

use crate::app_settings::models::{RelaySettings, RelaySettings as AppRelaySettings};
use crate::app_settings::store::{load_app_settings, save_app_settings};
use crate::bootstrap::BootstrapContext;
use crate::claude_accounts::{
    models::ClaudeAccountListItem, paths::ClaudeAccountPaths, service::ClaudeAccountService,
};
use crate::claude_usage::service::ClaudeUsageService;
use crate::codex_accounts::{
    models::CodexAccountListItem, paths::CodexAccountPaths, service::CodexAccountService,
};
use crate::codex_usage::service::CodexUsageService;
use crate::gemini_accounts::{
    models::GeminiAccountListItem, paths::GeminiAccountPaths, service::GeminiAccountService,
};
use crate::gemini_usage::service::GeminiUsageService;
use crate::relay::registry::{shared_runtime_status, stop_shared_runtime, RelayRegistryPaths};
use crate::relay::RelayRuntimeStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Provider {
    Codex,
    Claude,
    Gemini,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountRow {
    pub provider: Provider,
    pub id: String,
    pub email: String,
    pub is_active: bool,
    pub summary: String,
    pub needs_relogin: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentRow {
    pub provider: Provider,
    pub active_id: Option<String>,
    pub active_email: Option<String>,
    pub summary: String,
    pub needs_relogin: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshRow {
    pub provider: Provider,
    pub ok: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwitchOutcome {
    pub provider: Provider,
    pub id: String,
    pub email: String,
    pub already_active: bool,
}

#[derive(Debug, Clone)]
pub enum SwitchSelection {
    Id(String),
    Email(String),
}

#[derive(Debug, Clone)]
pub enum CliError {
    Usage(String),
    Environment(String),
    Provider(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) | Self::Environment(message) | Self::Provider(message) => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for CliError {}

pub struct CliFacade {
    context: BootstrapContext,
}

impl CliFacade {
    pub fn new(context: BootstrapContext) -> Self {
        Self { context }
    }

    pub fn list(&self, provider: Option<Provider>) -> Result<Vec<AccountRow>, CliError> {
        let mut rows = Vec::new();
        if should_include(provider, Provider::Codex) {
            rows.extend(
                CodexAccountService::with_process_runner(self.codex_paths())
                    .list_accounts()
                    .map_err(CliError::Provider)?
                    .into_iter()
                    .map(AccountRow::from_codex),
            );
        }
        if should_include(provider, Provider::Claude) {
            rows.extend(
                ClaudeAccountService::with_process_runner(self.claude_paths())
                    .list_accounts()
                    .map_err(CliError::Provider)?
                    .into_iter()
                    .map(AccountRow::from_claude),
            );
        }
        if should_include(provider, Provider::Gemini) {
            rows.extend(
                GeminiAccountService::with_process_runner(self.gemini_paths())
                    .list_accounts()
                    .map_err(CliError::Provider)?
                    .into_iter()
                    .map(AccountRow::from_gemini),
            );
        }
        Ok(rows)
    }

    pub fn current(&self, provider: Option<Provider>) -> Result<Vec<CurrentRow>, CliError> {
        let rows = self.list(provider)?;
        let mut current = Vec::new();

        for item in [Provider::Codex, Provider::Claude, Provider::Gemini] {
            if !should_include(provider, item) {
                continue;
            }

            let active = rows
                .iter()
                .find(|row| row.provider == item && row.is_active);
            current.push(CurrentRow {
                provider: item,
                active_id: active.map(|row| row.id.clone()),
                active_email: active.map(|row| row.email.clone()),
                summary: active
                    .map(|row| row.summary.clone())
                    .unwrap_or_else(|| "-".to_string()),
                needs_relogin: active.map(|row| row.needs_relogin).unwrap_or(false),
            });
        }

        Ok(current)
    }

    pub fn switch(
        &self,
        provider: Provider,
        selection: SwitchSelection,
    ) -> Result<SwitchOutcome, CliError> {
        match provider {
            Provider::Codex => self.switch_codex(selection),
            Provider::Claude => self.switch_claude(selection),
            Provider::Gemini => self.switch_gemini(selection),
        }
    }

    pub fn refresh(&self, provider: Option<Provider>) -> Result<Vec<RefreshRow>, CliError> {
        let mut rows = Vec::new();
        if should_include(provider, Provider::Codex) {
            let result = CodexUsageService::with_process_fetcher(self.codex_paths()).refresh_all();
            rows.push(RefreshRow {
                provider: Provider::Codex,
                ok: result.is_ok(),
                message: result.err(),
            });
        }
        if should_include(provider, Provider::Claude) {
            let result =
                ClaudeUsageService::with_process_fetchers(self.claude_paths()).refresh_all();
            rows.push(RefreshRow {
                provider: Provider::Claude,
                ok: result.is_ok(),
                message: result.err(),
            });
        }
        if should_include(provider, Provider::Gemini) {
            let result =
                GeminiUsageService::with_process_fetcher(self.gemini_paths()).refresh_all();
            rows.push(RefreshRow {
                provider: Provider::Gemini,
                ok: result.is_ok(),
                message: result.err(),
            });
        }
        Ok(rows)
    }

    pub fn relay_status(&self) -> Result<RelayRuntimeStatus, CliError> {
        let codex_paths = self.codex_paths();
        let settings = load_app_settings(&codex_paths).map_err(CliError::Environment)?;
        let registry_paths = RelayRegistryPaths::from_managed_root(&self.context.managed_root);
        Ok(shared_runtime_status(
            &settings.relay,
            &registry_paths,
            None,
        ))
    }

    pub fn relay_enable(&self, port: Option<u16>) -> Result<AppRelaySettings, CliError> {
        let codex_paths = self.codex_paths();
        let mut settings = load_app_settings(&codex_paths).map_err(CliError::Environment)?;
        settings.relay.enabled = true;
        if let Some(port) = port {
            settings.relay.port = port;
        }
        save_app_settings(&codex_paths, settings)
            .map(|settings| settings.relay)
            .map_err(CliError::Environment)
    }

    pub fn relay_disable(&self) -> Result<AppRelaySettings, CliError> {
        let codex_paths = self.codex_paths();
        let mut settings = load_app_settings(&codex_paths).map_err(CliError::Environment)?;
        settings.relay.enabled = false;
        save_app_settings(&codex_paths, settings)
            .map(|settings| settings.relay)
            .map_err(CliError::Environment)
    }

    pub fn relay_set_port(&self, port: u16) -> Result<AppRelaySettings, CliError> {
        let codex_paths = self.codex_paths();
        let mut settings = load_app_settings(&codex_paths).map_err(CliError::Environment)?;
        settings.relay.port = port;
        save_app_settings(&codex_paths, settings)
            .map(|settings| settings.relay)
            .map_err(CliError::Environment)
    }

    pub fn relay_settings(&self) -> Result<RelaySettings, CliError> {
        let codex_paths = self.codex_paths();
        load_app_settings(&codex_paths)
            .map(|settings| settings.relay)
            .map_err(CliError::Environment)
    }

    pub fn relay_stop(&self) -> Result<bool, CliError> {
        let registry_paths = RelayRegistryPaths::from_managed_root(&self.context.managed_root);
        stop_shared_runtime(&registry_paths).map_err(CliError::Environment)
    }

    fn switch_codex(&self, selection: SwitchSelection) -> Result<SwitchOutcome, CliError> {
        let service = CodexAccountService::with_process_runner(self.codex_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_codex(&accounts, selection)?;
        let already_active = selected.is_active;
        let id = selected.id.clone();
        let email = selected.email.clone();
        service.switch_account(&id).map_err(CliError::Provider)?;
        Ok(SwitchOutcome {
            provider: Provider::Codex,
            id,
            email,
            already_active,
        })
    }

    fn switch_claude(&self, selection: SwitchSelection) -> Result<SwitchOutcome, CliError> {
        let mut service = ClaudeAccountService::with_process_runner(self.claude_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_claude(&accounts, selection)?;
        let already_active = selected.is_active;
        let id = selected.id.clone();
        let email = selected.email.clone();
        service.switch_account(&id).map_err(CliError::Provider)?;
        Ok(SwitchOutcome {
            provider: Provider::Claude,
            id,
            email,
            already_active,
        })
    }

    fn switch_gemini(&self, selection: SwitchSelection) -> Result<SwitchOutcome, CliError> {
        let service = GeminiAccountService::with_process_runner(self.gemini_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_gemini(&accounts, selection)?;
        let already_active = selected.is_active;
        let id = selected.id.clone();
        let email = selected.email.clone();
        service.switch_account(&id).map_err(CliError::Provider)?;
        Ok(SwitchOutcome {
            provider: Provider::Gemini,
            id,
            email,
            already_active,
        })
    }

    fn codex_paths(&self) -> CodexAccountPaths {
        CodexAccountPaths::from_roots(
            self.context.managed_root.clone(),
            self.context.user_home.clone(),
        )
    }

    fn claude_paths(&self) -> ClaudeAccountPaths {
        ClaudeAccountPaths::from_roots(
            self.context.managed_root.clone(),
            self.context.user_home.clone(),
        )
    }

    fn gemini_paths(&self) -> GeminiAccountPaths {
        GeminiAccountPaths::from_roots(
            self.context.managed_root.clone(),
            self.context.user_home.clone(),
        )
    }
}

fn should_include(filter: Option<Provider>, provider: Provider) -> bool {
    filter.is_none() || filter == Some(provider)
}

fn select_codex(
    accounts: &[CodexAccountListItem],
    selection: SwitchSelection,
) -> Result<&CodexAccountListItem, CliError> {
    match selection {
        SwitchSelection::Id(id) => accounts
            .iter()
            .find(|account| account.id == id)
            .ok_or_else(|| CliError::Usage(format!("Codex account {id} not found"))),
        SwitchSelection::Email(email) => accounts
            .iter()
            .find(|account| account.email == email)
            .ok_or_else(|| CliError::Usage(format!("Codex account {email} not found"))),
    }
}

fn select_claude(
    accounts: &[ClaudeAccountListItem],
    selection: SwitchSelection,
) -> Result<&ClaudeAccountListItem, CliError> {
    match selection {
        SwitchSelection::Id(id) => accounts
            .iter()
            .find(|account| account.id == id)
            .ok_or_else(|| CliError::Usage(format!("Claude account {id} not found"))),
        SwitchSelection::Email(email) => accounts
            .iter()
            .find(|account| account.email == email)
            .ok_or_else(|| CliError::Usage(format!("Claude account {email} not found"))),
    }
}

fn select_gemini(
    accounts: &[GeminiAccountListItem],
    selection: SwitchSelection,
) -> Result<&GeminiAccountListItem, CliError> {
    match selection {
        SwitchSelection::Id(id) => accounts
            .iter()
            .find(|account| account.id == id)
            .ok_or_else(|| CliError::Usage(format!("Gemini account {id} not found"))),
        SwitchSelection::Email(email) => accounts
            .iter()
            .find(|account| account.email == email)
            .ok_or_else(|| CliError::Usage(format!("Gemini account {email} not found"))),
    }
}

impl AccountRow {
    fn from_codex(item: CodexAccountListItem) -> Self {
        Self {
            provider: Provider::Codex,
            id: item.id,
            email: item.email,
            is_active: item.is_active,
            summary: format_remaining("codex", item.five_hour_remaining_percent),
            needs_relogin: item.needs_relogin.unwrap_or(false),
        }
    }

    fn from_claude(item: ClaudeAccountListItem) -> Self {
        Self {
            provider: Provider::Claude,
            id: item.id,
            email: item.email,
            is_active: item.is_active,
            summary: format_remaining("claude", item.session_remaining_percent),
            needs_relogin: item.needs_relogin.unwrap_or(false),
        }
    }

    fn from_gemini(item: GeminiAccountListItem) -> Self {
        Self {
            provider: Provider::Gemini,
            id: item.id,
            email: item.email,
            is_active: item.is_active,
            summary: format_remaining("gemini", item.pro_remaining_percent),
            needs_relogin: item.needs_relogin.unwrap_or(false),
        }
    }
}

fn format_remaining(label: &str, remaining_percent: Option<u8>) -> String {
    match remaining_percent {
        Some(percent) => format!("{label} {percent}%"),
        None => "-".to_string(),
    }
}
