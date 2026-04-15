use serde::{Deserialize, Serialize};

use crate::app_settings::models::RelaySettings as AppRelaySettings;
use crate::app_settings::store::{load_app_settings, save_app_settings};
use crate::bootstrap::BootstrapContext;
use crate::claude_accounts::{
    cli::resolve_claude_binary,
    models::{ClaudeAccountListItem, StoredClaudeAccount},
    paths::ClaudeAccountPaths,
    service::ClaudeAccountService,
};
use crate::claude_usage::service::ClaudeUsageService;
use crate::codex_accounts::{
    cli::resolve_codex_binary,
    models::{CodexAccountListItem, StoredCodexAccount},
    paths::CodexAccountPaths,
    service::CodexAccountService,
};
use crate::codex_usage::service::CodexUsageService;
use crate::gemini_accounts::{
    cli::resolve_gemini_binary,
    models::{GeminiAccountListItem, StoredGeminiAccount},
    paths::GeminiAccountPaths,
    service::GeminiAccountService,
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
    pub label: Option<String>,
    pub is_active: bool,
    pub summary: String,
    pub needs_relogin: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentRow {
    pub provider: Provider,
    pub active_id: Option<String>,
    pub active_email: Option<String>,
    pub active_label: Option<String>,
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
pub struct HubPaths {
    pub managed_root: String,
    pub user_home: String,
    pub rows: Vec<PathRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathRow {
    pub scope: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub managed_root: String,
    pub user_home: String,
    pub relay: Option<RelayRuntimeStatus>,
    pub relay_error: Option<String>,
    pub providers: Vec<ProviderDoctorRow>,
    pub import_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderDoctorRow {
    pub provider: Provider,
    pub cli_path: Option<String>,
    pub account_count: usize,
    pub active_email: Option<String>,
    pub needs_relogin_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AddOutcome {
    pub provider: Provider,
    pub id: String,
    pub email: String,
    pub activated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SwitchOutcome {
    pub provider: Provider,
    pub id: String,
    pub email: String,
    pub already_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveOutcome {
    pub provider: Provider,
    pub id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelOutcome {
    pub provider: Provider,
    pub id: String,
    pub email: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadataExport {
    pub version: u8,
    pub accounts: Vec<AccountMetadataItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadataItem {
    pub provider: String,
    pub id: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportMetadataOutcome {
    pub imported_count: usize,
    pub skipped: Vec<ImportMetadataSkip>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportMetadataSkip {
    pub provider: String,
    pub id: String,
    pub email: String,
    pub reason: String,
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
                active_label: active.and_then(|row| row.label.clone()),
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

    pub fn remove(
        &self,
        provider: Provider,
        selection: SwitchSelection,
    ) -> Result<RemoveOutcome, CliError> {
        match provider {
            Provider::Codex => self.remove_codex(selection),
            Provider::Claude => self.remove_claude(selection),
            Provider::Gemini => self.remove_gemini(selection),
        }
    }

    pub fn label(
        &self,
        provider: Provider,
        selection: SwitchSelection,
        label: Option<String>,
    ) -> Result<LabelOutcome, CliError> {
        let label = label.and_then(normalize_label);
        match provider {
            Provider::Codex => self.label_codex(selection, label),
            Provider::Claude => self.label_claude(selection, label),
            Provider::Gemini => self.label_gemini(selection, label),
        }
    }

    pub fn export_metadata(&self) -> Result<AccountMetadataExport, CliError> {
        let accounts = self
            .list(None)?
            .into_iter()
            .map(|row| AccountMetadataItem {
                provider: provider_label(row.provider).to_string(),
                id: row.id,
                email: row.email,
                label: row.label,
            })
            .collect();

        Ok(AccountMetadataExport {
            version: 1,
            accounts,
        })
    }

    pub fn import_metadata(
        &self,
        metadata: AccountMetadataExport,
    ) -> Result<ImportMetadataOutcome, CliError> {
        if metadata.version != 1 {
            return Err(CliError::Usage(format!(
                "unsupported metadata export version {}",
                metadata.version
            )));
        }

        let mut imported_count = 0;
        let mut skipped = Vec::new();
        for item in metadata.accounts {
            let Some(provider) = parse_provider_label(&item.provider) else {
                skipped.push(skip_item(item, "unknown provider"));
                continue;
            };

            let accounts = self.list(Some(provider))?;
            let matched_id = accounts
                .iter()
                .find(|account| {
                    account.id == item.id || account.email.eq_ignore_ascii_case(&item.email)
                })
                .map(|account| account.id.clone());
            let Some(matched_id) = matched_id else {
                skipped.push(skip_item(item, "account not found"));
                continue;
            };

            self.label(
                provider,
                SwitchSelection::Id(matched_id),
                item.label.clone(),
            )?;
            imported_count += 1;
        }

        Ok(ImportMetadataOutcome {
            imported_count,
            skipped,
        })
    }

    pub fn add(&self, provider: Provider) -> Result<AddOutcome, CliError> {
        match provider {
            Provider::Codex => {
                let service = CodexAccountService::with_process_runner(self.codex_paths());
                service
                    .start_login()
                    .map(AddOutcome::from_codex)
                    .map_err(CliError::Provider)
            }
            Provider::Claude => {
                let mut service = ClaudeAccountService::with_process_runner(self.claude_paths());
                service
                    .start_login()
                    .map(AddOutcome::from_claude)
                    .map_err(CliError::Provider)
            }
            Provider::Gemini => {
                let service = GeminiAccountService::with_process_runner(self.gemini_paths());
                service
                    .start_login()
                    .map(AddOutcome::from_gemini)
                    .map_err(CliError::Provider)
            }
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

    pub fn paths(&self) -> HubPaths {
        let codex = self.codex_paths();
        let claude = self.claude_paths();
        let gemini = self.gemini_paths();
        HubPaths {
            managed_root: path_string(&self.context.managed_root),
            user_home: path_string(&self.context.user_home),
            rows: vec![
                path_row("codex", "data", &codex.codex_data_dir),
                path_row("codex", "accounts", &codex.account_index_path),
                path_row("codex", "usage", &codex.usage_snapshot_path),
                path_row("codex", "managed homes", &codex.managed_homes_dir),
                path_row("codex", "live auth", &codex.system_auth_path),
                path_row("claude", "data", &claude.claude_data_dir),
                path_row("claude", "accounts", &claude.metadata_index_path),
                path_row("claude", "usage", &claude.usage_snapshot_path),
                path_row("claude", "managed bundles", &claude.managed_bundle_dir),
                path_row(
                    "claude",
                    "live credentials",
                    &claude.system_credentials_path,
                ),
                path_row("claude", "live config", &claude.system_global_config_path),
                path_row("gemini", "data", &gemini.gemini_data_dir),
                path_row("gemini", "accounts", &gemini.account_index_path),
                path_row("gemini", "usage", &gemini.usage_snapshot_path),
                path_row("gemini", "managed homes", &gemini.managed_homes_dir),
                path_row("gemini", "live config", &gemini.system_gemini_dir),
            ],
        }
    }

    pub fn doctor(&self) -> DoctorReport {
        let (relay, relay_error) = match self.relay_status() {
            Ok(status) => (Some(status), None),
            Err(error) => (None, Some(error.to_string())),
        };

        DoctorReport {
            managed_root: path_string(&self.context.managed_root),
            user_home: path_string(&self.context.user_home),
            relay,
            relay_error,
            providers: [Provider::Codex, Provider::Claude, Provider::Gemini]
                .into_iter()
                .map(|provider| self.provider_doctor(provider))
                .collect(),
            import_warnings: self.context.import_warnings.clone(),
        }
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

    fn remove_codex(&self, selection: SwitchSelection) -> Result<RemoveOutcome, CliError> {
        let service = CodexAccountService::with_process_runner(self.codex_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_codex(&accounts, selection)?;
        ensure_not_active(Provider::Codex, selected.is_active, &selected.email)?;
        let id = selected.id.clone();
        let email = selected.email.clone();
        service.delete_account(&id).map_err(CliError::Provider)?;
        Ok(RemoveOutcome {
            provider: Provider::Codex,
            id,
            email,
        })
    }

    fn remove_claude(&self, selection: SwitchSelection) -> Result<RemoveOutcome, CliError> {
        let mut service = ClaudeAccountService::with_process_runner(self.claude_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_claude(&accounts, selection)?;
        ensure_not_active(Provider::Claude, selected.is_active, &selected.email)?;
        let id = selected.id.clone();
        let email = selected.email.clone();
        service.delete_account(&id).map_err(CliError::Provider)?;
        Ok(RemoveOutcome {
            provider: Provider::Claude,
            id,
            email,
        })
    }

    fn remove_gemini(&self, selection: SwitchSelection) -> Result<RemoveOutcome, CliError> {
        let service = GeminiAccountService::with_process_runner(self.gemini_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_gemini(&accounts, selection)?;
        ensure_not_active(Provider::Gemini, selected.is_active, &selected.email)?;
        let id = selected.id.clone();
        let email = selected.email.clone();
        service.delete_account(&id).map_err(CliError::Provider)?;
        Ok(RemoveOutcome {
            provider: Provider::Gemini,
            id,
            email,
        })
    }

    fn label_codex(
        &self,
        selection: SwitchSelection,
        label: Option<String>,
    ) -> Result<LabelOutcome, CliError> {
        let service = CodexAccountService::with_process_runner(self.codex_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_codex(&accounts, selection)?;
        let saved = service
            .set_label(&selected.id, label)
            .map_err(CliError::Provider)?;
        Ok(LabelOutcome {
            provider: Provider::Codex,
            id: saved.id,
            email: saved.email,
            label: saved.label,
        })
    }

    fn label_claude(
        &self,
        selection: SwitchSelection,
        label: Option<String>,
    ) -> Result<LabelOutcome, CliError> {
        let mut service = ClaudeAccountService::with_process_runner(self.claude_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_claude(&accounts, selection)?;
        let saved = service
            .set_label(&selected.id, label)
            .map_err(CliError::Provider)?;
        Ok(LabelOutcome {
            provider: Provider::Claude,
            id: saved.id,
            email: saved.email,
            label: saved.label,
        })
    }

    fn label_gemini(
        &self,
        selection: SwitchSelection,
        label: Option<String>,
    ) -> Result<LabelOutcome, CliError> {
        let service = GeminiAccountService::with_process_runner(self.gemini_paths());
        let accounts = service.list_accounts().map_err(CliError::Provider)?;
        let selected = select_gemini(&accounts, selection)?;
        let saved = service
            .set_label(&selected.id, label)
            .map_err(CliError::Provider)?;
        Ok(LabelOutcome {
            provider: Provider::Gemini,
            id: saved.id,
            email: saved.email,
            label: saved.label,
        })
    }

    fn provider_doctor(&self, provider: Provider) -> ProviderDoctorRow {
        let cli_path = provider_cli_path(provider).map(|path| path_string(&path));
        let mut issues = Vec::new();
        if cli_path.is_none() {
            issues.push(format!("{} CLI not found", provider_title(provider)));
        }

        match self.list(Some(provider)) {
            Ok(accounts) => {
                let needs_relogin_count = accounts
                    .iter()
                    .filter(|account| account.needs_relogin)
                    .count();
                if needs_relogin_count > 0 {
                    issues.push(format!("{needs_relogin_count} account(s) need relogin"));
                }

                ProviderDoctorRow {
                    provider,
                    cli_path,
                    account_count: accounts.len(),
                    active_email: accounts
                        .iter()
                        .find(|account| account.is_active)
                        .map(|account| account.email.clone()),
                    needs_relogin_count,
                    issues,
                }
            }
            Err(error) => {
                issues.push(format!("failed to inspect accounts: {error}"));
                ProviderDoctorRow {
                    provider,
                    cli_path,
                    account_count: 0,
                    active_email: None,
                    needs_relogin_count: 0,
                    issues,
                }
            }
        }
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

fn normalize_label(label: String) -> Option<String> {
    let trimmed = label.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn ensure_not_active(provider: Provider, is_active: bool, email: &str) -> Result<(), CliError> {
    if is_active {
        Err(CliError::Usage(format!(
            "Cannot remove active {} account {email}; switch to another account first",
            provider_title(provider)
        )))
    } else {
        Ok(())
    }
}

fn path_row(scope: &str, name: &str, path: &std::path::Path) -> PathRow {
    PathRow {
        scope: scope.to_string(),
        name: name.to_string(),
        path: path_string(path),
    }
}

fn path_string(path: &std::path::Path) -> String {
    path.display().to_string()
}

fn provider_cli_path(provider: Provider) -> Option<std::path::PathBuf> {
    match provider {
        Provider::Codex => resolve_codex_binary(),
        Provider::Claude => resolve_claude_binary(),
        Provider::Gemini => resolve_gemini_binary(),
    }
}

fn provider_title(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "Codex",
        Provider::Claude => "Claude",
        Provider::Gemini => "Gemini",
    }
}

fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "codex",
        Provider::Claude => "claude",
        Provider::Gemini => "gemini",
    }
}

fn parse_provider_label(provider: &str) -> Option<Provider> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "codex" => Some(Provider::Codex),
        "claude" => Some(Provider::Claude),
        "gemini" => Some(Provider::Gemini),
        _ => None,
    }
}

fn skip_item(item: AccountMetadataItem, reason: &str) -> ImportMetadataSkip {
    ImportMetadataSkip {
        provider: item.provider,
        id: item.id,
        email: item.email,
        reason: reason.to_string(),
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
            label: item.label,
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
            label: item.label,
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
            label: item.label,
            is_active: item.is_active,
            summary: format_remaining("gemini", item.pro_remaining_percent),
            needs_relogin: item.needs_relogin.unwrap_or(false),
        }
    }
}

impl AddOutcome {
    fn from_codex(account: StoredCodexAccount) -> Self {
        Self {
            provider: Provider::Codex,
            id: account.id,
            email: account.email,
            activated: false,
        }
    }

    fn from_claude(account: StoredClaudeAccount) -> Self {
        Self {
            provider: Provider::Claude,
            id: account.id,
            email: account.email,
            activated: false,
        }
    }

    fn from_gemini(account: StoredGeminiAccount) -> Self {
        Self {
            provider: Provider::Gemini,
            id: account.id,
            email: account.email,
            activated: false,
        }
    }
}

fn format_remaining(label: &str, remaining_percent: Option<u8>) -> String {
    match remaining_percent {
        Some(percent) => format!("{label} {percent}%"),
        None => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude_accounts::models::StoredClaudeAccount;
    use crate::codex_accounts::models::StoredCodexAccount;
    use crate::gemini_accounts::models::StoredGeminiAccount;

    #[test]
    fn add_outcome_from_codex_account_marks_not_activated() {
        let outcome =
            AddOutcome::from_codex(StoredCodexAccount::new_for_tests("user@example.com", None));

        assert_eq!(outcome.provider, Provider::Codex);
        assert_eq!(outcome.id, "test-user@example.com");
        assert_eq!(outcome.email, "user@example.com");
        assert!(!outcome.activated);
    }

    #[test]
    fn add_outcome_from_claude_account_marks_not_activated() {
        let outcome =
            AddOutcome::from_claude(StoredClaudeAccount::new_for_tests("user@example.com", None));

        assert_eq!(outcome.provider, Provider::Claude);
        assert_eq!(outcome.id, "test-user@example.com");
        assert_eq!(outcome.email, "user@example.com");
        assert!(!outcome.activated);
    }

    #[test]
    fn add_outcome_from_gemini_account_marks_not_activated() {
        let outcome =
            AddOutcome::from_gemini(StoredGeminiAccount::new_for_tests("user@example.com", None));

        assert_eq!(outcome.provider, Provider::Gemini);
        assert_eq!(outcome.id, "test-user@example.com");
        assert_eq!(outcome.email, "user@example.com");
        assert!(!outcome.activated);
    }
}
