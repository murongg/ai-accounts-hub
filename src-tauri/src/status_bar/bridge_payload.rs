use serde::{Deserialize, Serialize};

use crate::claude_accounts::models::ClaudeAccountListItem;
use crate::codex_accounts::models::CodexAccountListItem;
use crate::gemini_accounts::models::GeminiAccountListItem;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusBarTab {
    Overview,
    Codex,
    Claude,
    Gemini,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeMetricPayload {
    pub title: String,
    pub percent: u8,
    pub left_text: String,
    pub reset_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeProviderPayload {
    pub id: String,
    pub provider_id: String,
    pub provider_title: String,
    pub email: String,
    pub subtitle: String,
    pub plan: Option<String>,
    pub is_active: bool,
    pub needs_relogin: bool,
    pub metrics: Vec<BridgeMetricPayload>,
    pub switch_account_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatusItemProgressPayload {
    pub provider_id: String,
    pub percent: u8,
    pub needs_relogin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgePayload {
    pub selected_tab: StatusBarTab,
    pub sections: Vec<BridgeProviderPayload>,
    pub status_item_progress: Option<BridgeStatusItemProgressPayload>,
}

pub fn build_bridge_payload(
    selected_tab: StatusBarTab,
    codex_accounts: Vec<CodexAccountListItem>,
    claude_accounts: Vec<ClaudeAccountListItem>,
    gemini_accounts: Vec<GeminiAccountListItem>,
    now_ms: i64,
) -> BridgePayload {
    let status_item_progress = build_status_item_progress(
        selected_tab,
        &codex_accounts,
        &claude_accounts,
        &gemini_accounts,
    );
    let mut codex_sections = build_codex_sections(codex_accounts, now_ms);
    let mut claude_sections = build_claude_sections(claude_accounts, now_ms);
    let mut gemini_sections = build_gemini_sections(gemini_accounts, now_ms);

    let sections = match selected_tab {
        StatusBarTab::Overview => {
            let mut overview = Vec::new();
            if let Some(active_codex) = codex_sections
                .iter()
                .find(|section| section.is_active)
                .cloned()
                .or_else(|| codex_sections.first().cloned())
            {
                overview.push(active_codex);
            }
            if let Some(active_claude) = claude_sections
                .iter()
                .find(|section| section.is_active)
                .cloned()
                .or_else(|| claude_sections.first().cloned())
            {
                overview.push(active_claude);
            }
            if let Some(active_gemini) = gemini_sections
                .iter()
                .find(|section| section.is_active)
                .cloned()
                .or_else(|| gemini_sections.first().cloned())
            {
                overview.push(active_gemini);
            }
            overview
        }
        StatusBarTab::Codex => {
            sort_sections(&mut codex_sections);
            codex_sections
        }
        StatusBarTab::Claude => {
            sort_sections(&mut claude_sections);
            claude_sections
        }
        StatusBarTab::Gemini => {
            sort_sections(&mut gemini_sections);
            gemini_sections
        }
    };

    BridgePayload {
        selected_tab,
        sections,
        status_item_progress,
    }
}

fn build_status_item_progress(
    selected_tab: StatusBarTab,
    codex_accounts: &[CodexAccountListItem],
    claude_accounts: &[ClaudeAccountListItem],
    gemini_accounts: &[GeminiAccountListItem],
) -> Option<BridgeStatusItemProgressPayload> {
    match selected_tab {
        StatusBarTab::Overview => None,
        StatusBarTab::Codex => codex_accounts
            .iter()
            .find(|account| account.is_active)
            .and_then(|account| {
                if account.needs_relogin.unwrap_or(false) {
                    None
                } else {
                    account.five_hour_remaining_percent.map(|percent| {
                        BridgeStatusItemProgressPayload {
                            provider_id: "codex".to_string(),
                            percent,
                            needs_relogin: false,
                        }
                    })
                }
            }),
        StatusBarTab::Claude => claude_accounts
            .iter()
            .find(|account| account.is_active)
            .and_then(|account| {
                if account.needs_relogin.unwrap_or(false) {
                    None
                } else {
                    account.session_remaining_percent.map(|percent| {
                        BridgeStatusItemProgressPayload {
                            provider_id: "claude".to_string(),
                            percent,
                            needs_relogin: false,
                        }
                    })
                }
            }),
        StatusBarTab::Gemini => gemini_accounts
            .iter()
            .find(|account| account.is_active)
            .and_then(|account| {
                if account.needs_relogin.unwrap_or(false) {
                    None
                } else {
                    account
                        .pro_remaining_percent
                        .map(|percent| BridgeStatusItemProgressPayload {
                            provider_id: "gemini".to_string(),
                            percent,
                            needs_relogin: false,
                        })
                }
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codex_account(
        id: &str,
        is_active: bool,
        five_hour_remaining_percent: Option<u8>,
        needs_relogin: Option<bool>,
    ) -> CodexAccountListItem {
        CodexAccountListItem {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            plan: Some("Plus".to_string()),
            account_id: Some(format!("acct-{id}")),
            is_active,
            last_authenticated_at: "0".to_string(),
            five_hour_remaining_percent,
            weekly_remaining_percent: Some(91),
            five_hour_refresh_at: Some("1735689600000".to_string()),
            weekly_refresh_at: Some("1736294400000".to_string()),
            last_synced_at: Some("1735686000000".to_string()),
            last_sync_error: None,
            credits_balance: None,
            needs_relogin,
        }
    }

    fn claude_account(
        id: &str,
        is_active: bool,
        session_remaining_percent: Option<u8>,
        needs_relogin: Option<bool>,
    ) -> ClaudeAccountListItem {
        ClaudeAccountListItem {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            display_name: Some("Claude User".to_string()),
            plan: Some("Pro".to_string()),
            account_hint: Some(format!("hint-{id}")),
            is_active,
            last_authenticated_at: "0".to_string(),
            session_remaining_percent,
            session_refresh_at: Some("1735689600000".to_string()),
            weekly_remaining_percent: Some(70),
            weekly_refresh_at: Some("1736294400000".to_string()),
            model_weekly_label: Some("Opus Weekly".to_string()),
            model_weekly_remaining_percent: Some(54),
            model_weekly_refresh_at: Some("1736294400000".to_string()),
            last_synced_at: Some("1735686000000".to_string()),
            last_sync_error: None,
            needs_relogin,
        }
    }

    fn gemini_account(
        id: &str,
        is_active: bool,
        pro_remaining_percent: Option<u8>,
        needs_relogin: Option<bool>,
    ) -> GeminiAccountListItem {
        GeminiAccountListItem {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            subject: Some(format!("subject-{id}")),
            auth_type: Some("oauth-personal".to_string()),
            plan: Some("Pro".to_string()),
            is_active,
            last_authenticated_at: "0".to_string(),
            pro_remaining_percent,
            flash_remaining_percent: Some(48),
            flash_lite_remaining_percent: Some(37),
            pro_refresh_at: Some("1735689600000".to_string()),
            flash_refresh_at: Some("1735689600000".to_string()),
            flash_lite_refresh_at: Some("1735689600000".to_string()),
            last_synced_at: Some("1735686000000".to_string()),
            last_sync_error: None,
            needs_relogin,
        }
    }

    #[test]
    fn build_bridge_payload_uses_active_codex_session_quota_for_status_item_progress() {
        let payload = build_bridge_payload(
            StatusBarTab::Codex,
            vec![
                codex_account("inactive", false, Some(12), Some(false)),
                codex_account("active", true, Some(72), Some(false)),
            ],
            vec![],
            vec![],
            0,
        );

        assert_eq!(
            payload.status_item_progress,
            Some(BridgeStatusItemProgressPayload {
                provider_id: "codex".to_string(),
                percent: 72,
                needs_relogin: false,
            })
        );
    }

    #[test]
    fn build_bridge_payload_returns_none_when_active_claude_account_requires_relogin() {
        let payload = build_bridge_payload(
            StatusBarTab::Claude,
            vec![],
            vec![claude_account("active", true, Some(82), Some(true))],
            vec![],
            0,
        );

        assert_eq!(payload.status_item_progress, None);
    }

    #[test]
    fn build_bridge_payload_uses_gemini_pro_percent_for_status_item_progress() {
        let payload = build_bridge_payload(
            StatusBarTab::Gemini,
            vec![],
            vec![],
            vec![gemini_account("active", true, Some(61), Some(false))],
            0,
        );

        assert_eq!(
            payload.status_item_progress,
            Some(BridgeStatusItemProgressPayload {
                provider_id: "gemini".to_string(),
                percent: 61,
                needs_relogin: false,
            })
        );
    }

    #[test]
    fn build_bridge_payload_returns_none_when_visible_provider_has_no_active_account() {
        let payload = build_bridge_payload(
            StatusBarTab::Codex,
            vec![codex_account("inactive", false, Some(33), Some(false))],
            vec![],
            vec![],
            0,
        );

        assert_eq!(payload.status_item_progress, None);
    }
}

fn build_codex_sections(
    accounts: Vec<CodexAccountListItem>,
    now_ms: i64,
) -> Vec<BridgeProviderPayload> {
    accounts
        .into_iter()
        .map(|account| {
            let needs_relogin = account.needs_relogin.unwrap_or(false);
            let metrics = if needs_relogin {
                Vec::new()
            } else {
                [
                    account
                        .five_hour_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Session".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.five_hour_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                    account
                        .weekly_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Weekly".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.weekly_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                ]
                .into_iter()
                .flatten()
                .collect()
            };

            BridgeProviderPayload {
                id: format!("codex:{}", account.id),
                provider_id: "codex".to_string(),
                provider_title: "Codex".to_string(),
                email: account.email,
                subtitle: section_subtitle(
                    needs_relogin,
                    account.last_synced_at.as_deref(),
                    now_ms,
                ),
                plan: account.plan,
                is_active: account.is_active,
                needs_relogin,
                metrics,
                switch_account_id: (!account.is_active).then_some(account.id),
            }
        })
        .collect()
}

fn build_gemini_sections(
    accounts: Vec<GeminiAccountListItem>,
    now_ms: i64,
) -> Vec<BridgeProviderPayload> {
    accounts
        .into_iter()
        .map(|account| {
            let needs_relogin = account.needs_relogin.unwrap_or(false);
            let metrics = if needs_relogin {
                Vec::new()
            } else {
                [
                    account
                        .pro_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Pro".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.pro_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                    account
                        .flash_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Flash".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.flash_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                    account
                        .flash_lite_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Flash Lite".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.flash_lite_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                ]
                .into_iter()
                .flatten()
                .collect()
            };

            BridgeProviderPayload {
                id: format!("gemini:{}", account.id),
                provider_id: "gemini".to_string(),
                provider_title: "Gemini".to_string(),
                email: account.email,
                subtitle: section_subtitle(
                    needs_relogin,
                    account.last_synced_at.as_deref(),
                    now_ms,
                ),
                plan: account.plan,
                is_active: account.is_active,
                needs_relogin,
                metrics,
                switch_account_id: (!account.is_active).then_some(account.id),
            }
        })
        .collect()
}

fn build_claude_sections(
    accounts: Vec<ClaudeAccountListItem>,
    now_ms: i64,
) -> Vec<BridgeProviderPayload> {
    accounts
        .into_iter()
        .map(|account| {
            let needs_relogin = account.needs_relogin.unwrap_or(false);
            let metrics = if needs_relogin {
                Vec::new()
            } else {
                [
                    account
                        .session_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Session".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.session_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                    account
                        .weekly_remaining_percent
                        .map(|percent| BridgeMetricPayload {
                            title: "Weekly".to_string(),
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(account.weekly_refresh_at.as_deref(), now_ms)
                            ),
                        }),
                    account
                        .model_weekly_remaining_percent
                        .zip(account.model_weekly_label.clone())
                        .map(|(percent, title)| BridgeMetricPayload {
                            title,
                            percent,
                            left_text: format!("{percent}% left"),
                            reset_text: format!(
                                "Resets in {}",
                                format_countdown(
                                    account.model_weekly_refresh_at.as_deref(),
                                    now_ms
                                )
                            ),
                        }),
                ]
                .into_iter()
                .flatten()
                .collect()
            };

            BridgeProviderPayload {
                id: format!("claude:{}", account.id),
                provider_id: "claude".to_string(),
                provider_title: "Claude".to_string(),
                email: account.email,
                subtitle: section_subtitle(
                    needs_relogin,
                    account.last_synced_at.as_deref(),
                    now_ms,
                ),
                plan: account.plan,
                is_active: account.is_active,
                needs_relogin,
                metrics,
                switch_account_id: (!account.is_active).then_some(account.id),
            }
        })
        .collect()
}

fn sort_sections(sections: &mut [BridgeProviderPayload]) {
    sections.sort_by(|left, right| match (left.is_active, right.is_active) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });
}

fn section_subtitle(needs_relogin: bool, last_synced_at: Option<&str>, now_ms: i64) -> String {
    if needs_relogin {
        return "Re-login required".to_string();
    }

    match relative_updated_label(last_synced_at, now_ms) {
        Some(label) => format!("Updated {label}"),
        None => "Updated recently".to_string(),
    }
}

fn relative_updated_label(raw: Option<&str>, now_ms: i64) -> Option<String> {
    let seconds = raw?.parse::<i64>().ok()?;
    if seconds <= 0 {
        return None;
    }

    let diff_minutes = ((now_ms - seconds * 1000).max(0)) / 60_000;
    if diff_minutes <= 0 {
        Some("just now".to_string())
    } else if diff_minutes < 60 {
        Some(format!("{diff_minutes}m ago"))
    } else {
        let diff_hours = diff_minutes / 60;
        if diff_hours < 24 {
            Some(format!("{diff_hours}h ago"))
        } else {
            Some(format!("{}d ago", diff_hours / 24))
        }
    }
}

fn format_countdown(raw: Option<&str>, now_ms: i64) -> String {
    let Some(refresh_at_ms) = resolve_refresh_at_ms(raw) else {
        return "--".to_string();
    };

    let diff_ms = (refresh_at_ms - now_ms).max(0);
    let total_minutes = diff_ms / 60_000;

    if total_minutes < 60 {
        return format!("{}m", total_minutes.max(1));
    }

    let total_hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if total_hours < 24 {
        if minutes == 0 {
            return format!("{total_hours}h");
        }
        return format!("{total_hours}h {minutes}m");
    }

    let days = total_hours / 24;
    let hours = total_hours % 24;
    let mut parts = vec![format!("{days}d")];
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    parts.join(" ")
}

fn resolve_refresh_at_ms(raw: Option<&str>) -> Option<i64> {
    let raw = raw?;

    if let Ok(seconds) = raw.parse::<i64>() {
        if seconds > 0 {
            return Some(seconds * 1000);
        }
    }

    let parsed = chrono_like_parse(raw)?;
    Some(parsed)
}

fn chrono_like_parse(raw: &str) -> Option<i64> {
    let parsed =
        time::OffsetDateTime::parse(raw, &time::format_description::well_known::Rfc3339).ok()?;
    Some(parsed.unix_timestamp() * 1000)
}
