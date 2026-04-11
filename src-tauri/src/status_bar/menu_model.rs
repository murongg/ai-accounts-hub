use crate::claude_accounts::models::ClaudeAccountListItem;
use crate::codex_accounts::models::CodexAccountListItem;
use crate::gemini_accounts::models::GeminiAccountListItem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuProvider {
    Codex,
    Claude,
    Gemini,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    SelectProvider(MenuProvider),
    SwitchAccount(MenuProvider, String),
    RefreshSelectedProvider,
    OpenMainWindow,
    Quit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuAccountState {
    pub id: String,
    pub email: String,
    pub plan: String,
    pub quota_summary: Option<String>,
    pub is_active: bool,
    pub status_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderMenuState {
    pub selected_provider: MenuProvider,
    pub accounts: Vec<MenuAccountState>,
}

pub fn build_provider_menu_state(
    selected_provider: MenuProvider,
    codex_accounts: Vec<CodexAccountListItem>,
    claude_accounts: Vec<ClaudeAccountListItem>,
    gemini_accounts: Vec<GeminiAccountListItem>,
) -> ProviderMenuState {
    let accounts = match selected_provider {
        MenuProvider::Codex => sort_menu_accounts(
            codex_accounts
                .into_iter()
                .map(|account| {
                    let quota_summary = build_codex_quota_summary(&account);
                    let status_label = if account.needs_relogin.unwrap_or(false) {
                        "Re-login required".to_string()
                    } else {
                        "Healthy".to_string()
                    };

                    MenuAccountState {
                        id: account.id,
                        email: account.email,
                        plan: account
                            .plan
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                        quota_summary,
                        is_active: account.is_active,
                        status_label,
                    }
                })
                .collect(),
        ),
        MenuProvider::Claude => sort_menu_accounts(
            claude_accounts
                .into_iter()
                .map(|account| {
                    let quota_summary = build_claude_quota_summary(&account);
                    let status_label = if account.needs_relogin.unwrap_or(false) {
                        "Re-login required".to_string()
                    } else {
                        "Healthy".to_string()
                    };

                    MenuAccountState {
                        id: account.id,
                        email: account.email,
                        plan: account.plan.unwrap_or_else(|| "Unknown".to_string()),
                        quota_summary,
                        is_active: account.is_active,
                        status_label,
                    }
                })
                .collect(),
        ),
        MenuProvider::Gemini => sort_menu_accounts(
            gemini_accounts
                .into_iter()
                .map(|account| {
                    let quota_summary = build_gemini_quota_summary(&account);
                    let status_label = if account.needs_relogin.unwrap_or(false) {
                        "Re-login required".to_string()
                    } else {
                        "Healthy".to_string()
                    };

                    MenuAccountState {
                        id: account.id,
                        email: account.email,
                        plan: account
                            .plan
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string()),
                        quota_summary,
                        is_active: account.is_active,
                        status_label,
                    }
                })
                .collect(),
        ),
    };

    ProviderMenuState {
        selected_provider,
        accounts,
    }
}

pub fn parse_menu_action(id: &str) -> Option<MenuAction> {
    match id {
        "provider:codex" => Some(MenuAction::SelectProvider(MenuProvider::Codex)),
        "provider:claude" => Some(MenuAction::SelectProvider(MenuProvider::Claude)),
        "provider:gemini" => Some(MenuAction::SelectProvider(MenuProvider::Gemini)),
        "refresh" => Some(MenuAction::RefreshSelectedProvider),
        "open-main" => Some(MenuAction::OpenMainWindow),
        "quit" => Some(MenuAction::Quit),
        _ => parse_switch_action(id),
    }
}

fn parse_switch_action(id: &str) -> Option<MenuAction> {
    let remainder = id.strip_prefix("switch:")?;
    let (provider, account_id) = remainder.split_once(':')?;
    if account_id.is_empty() {
        return None;
    }

    let provider = match provider {
        "codex" => MenuProvider::Codex,
        "claude" => MenuProvider::Claude,
        "gemini" => MenuProvider::Gemini,
        _ => return None,
    };

    Some(MenuAction::SwitchAccount(provider, account_id.to_string()))
}

fn sort_menu_accounts(accounts: Vec<MenuAccountState>) -> Vec<MenuAccountState> {
    let mut indexed: Vec<(usize, MenuAccountState)> = accounts.into_iter().enumerate().collect();
    indexed.sort_by(|left, right| match (left.1.is_active, right.1.is_active) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => left.0.cmp(&right.0),
    });
    indexed.into_iter().map(|(_, account)| account).collect()
}

fn build_codex_quota_summary(account: &CodexAccountListItem) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(percent) = account.five_hour_remaining_percent {
        parts.push(format!("5h {percent}%"));
    }

    if let Some(percent) = account.weekly_remaining_percent {
        parts.push(format!("Week {percent}%"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" • "))
    }
}

fn build_gemini_quota_summary(account: &GeminiAccountListItem) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(percent) = account.pro_remaining_percent {
        parts.push(format!("Pro {percent}%"));
    }

    if let Some(percent) = account.flash_remaining_percent {
        parts.push(format!("Flash {percent}%"));
    }

    if let Some(percent) = account.flash_lite_remaining_percent {
        parts.push(format!("Lite {percent}%"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" • "))
    }
}

fn build_claude_quota_summary(account: &ClaudeAccountListItem) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(percent) = account.session_remaining_percent {
        parts.push(format!("Session {percent}%"));
    }

    if let Some(percent) = account.weekly_remaining_percent {
        parts.push(format!("Week {percent}%"));
    }

    if let Some(percent) = account.model_weekly_remaining_percent {
        let label = account
            .model_weekly_label
            .as_deref()
            .map(|value| value.replace(" Weekly", ""))
            .unwrap_or_else(|| "Model".to_string());
        parts.push(format!("{label} {percent}%"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" • "))
    }
}
