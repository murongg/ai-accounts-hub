use crate::claude_accounts::models::ClaudeAccountListItem;
use crate::codex_accounts::models::CodexAccountListItem;
use crate::gemini_accounts::models::GeminiAccountListItem;

pub fn select_codex_auto_switch_target(accounts: &[CodexAccountListItem]) -> Option<String> {
    select_auto_switch_target(
        accounts,
        |account| account.id.as_str(),
        |account| account.is_active,
        |account| account.needs_relogin.unwrap_or(false),
        |account| account.five_hour_remaining_percent,
    )
}

pub fn select_claude_auto_switch_target(accounts: &[ClaudeAccountListItem]) -> Option<String> {
    select_auto_switch_target(
        accounts,
        |account| account.id.as_str(),
        |account| account.is_active,
        |account| account.needs_relogin.unwrap_or(false),
        |account| account.session_remaining_percent,
    )
}

pub fn select_gemini_auto_switch_target(accounts: &[GeminiAccountListItem]) -> Option<String> {
    select_auto_switch_target(
        accounts,
        |account| account.id.as_str(),
        |account| account.is_active,
        |account| account.needs_relogin.unwrap_or(false),
        |account| account.pro_remaining_percent,
    )
}

pub fn select_auto_switch_target<T, Id, IsActive, NeedsRelogin, PrimaryQuota>(
    accounts: &[T],
    id: Id,
    is_active: IsActive,
    needs_relogin: NeedsRelogin,
    primary_quota_percent: PrimaryQuota,
) -> Option<String>
where
    Id: Fn(&T) -> &str,
    IsActive: Fn(&T) -> bool,
    NeedsRelogin: Fn(&T) -> bool,
    PrimaryQuota: Fn(&T) -> Option<u8>,
{
    let active_account = accounts.iter().find(|account| is_active(account))?;
    let active_is_unusable =
        needs_relogin(active_account) || primary_quota_percent(active_account) == Some(0);

    if !active_is_unusable {
        return None;
    }

    accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| !is_active(account) && !needs_relogin(account))
        .filter_map(|(index, account)| {
            let quota = primary_quota_percent(account)?;
            (quota > 0).then(|| (index, quota, id(account).to_string()))
        })
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(_, _, account_id)| account_id)
}

#[cfg(test)]
mod tests {
    use super::{
        select_auto_switch_target, select_claude_auto_switch_target,
        select_codex_auto_switch_target, select_gemini_auto_switch_target,
    };
    use crate::claude_accounts::models::ClaudeAccountListItem;
    use crate::codex_accounts::models::CodexAccountListItem;
    use crate::gemini_accounts::models::GeminiAccountListItem;

    #[derive(Clone, Debug)]
    struct Account {
        id: &'static str,
        is_active: bool,
        needs_relogin: bool,
        primary_quota_percent: Option<u8>,
    }

    fn account(
        id: &'static str,
        is_active: bool,
        needs_relogin: bool,
        primary_quota_percent: Option<u8>,
    ) -> Account {
        Account {
            id,
            is_active,
            needs_relogin,
            primary_quota_percent,
        }
    }

    fn select(accounts: &[Account]) -> Option<String> {
        select_auto_switch_target(
            accounts,
            |account| account.id,
            |account| account.is_active,
            |account| account.needs_relogin,
            |account| account.primary_quota_percent,
        )
    }

    #[test]
    fn selects_highest_quota_candidate_when_active_account_is_used_up() {
        let accounts = vec![
            account("active", true, false, Some(0)),
            account("low", false, false, Some(12)),
            account("missing", false, false, None),
            account("high", false, false, Some(91)),
            account("relogin", false, true, Some(100)),
        ];

        assert_eq!(select(&accounts), Some("high".to_string()));
    }

    #[test]
    fn does_not_switch_when_active_quota_is_still_available_or_unknown() {
        let available = vec![
            account("active", true, false, Some(1)),
            account("candidate", false, false, Some(99)),
        ];
        let unknown = vec![
            account("active", true, false, None),
            account("candidate", false, false, Some(99)),
        ];

        assert_eq!(select(&available), None);
        assert_eq!(select(&unknown), None);
    }

    #[test]
    fn switches_away_from_active_account_that_needs_relogin() {
        let accounts = vec![
            account("active", true, true, Some(100)),
            account("candidate", false, false, Some(42)),
        ];

        assert_eq!(select(&accounts), Some("candidate".to_string()));
    }

    #[test]
    fn ignores_candidates_without_positive_primary_quota() {
        let accounts = vec![
            account("active", true, false, Some(0)),
            account("zero", false, false, Some(0)),
            account("missing", false, false, None),
            account("relogin", false, true, Some(100)),
        ];

        assert_eq!(select(&accounts), None);
    }

    #[test]
    fn codex_auto_switch_uses_five_hour_quota_as_primary() {
        let accounts = vec![
            codex_account("active", true, Some(0), Some(0)),
            codex_account("weekly-high", false, Some(12), Some(99)),
            codex_account("five-hour-high", false, Some(91), Some(1)),
        ];

        assert_eq!(
            select_codex_auto_switch_target(&accounts),
            Some("five-hour-high".to_string())
        );
    }

    #[test]
    fn claude_and_gemini_auto_switch_use_provider_primary_quotas() {
        let claude_accounts = vec![
            claude_account("active", true, Some(0), Some(0)),
            claude_account("weekly-high", false, Some(18), Some(99)),
            claude_account("session-high", false, Some(72), Some(10)),
        ];
        let gemini_accounts = vec![
            gemini_account("active", true, Some(0), Some(0)),
            gemini_account("flash-high", false, Some(11), Some(99)),
            gemini_account("pro-high", false, Some(88), Some(10)),
        ];

        assert_eq!(
            select_claude_auto_switch_target(&claude_accounts),
            Some("session-high".to_string())
        );
        assert_eq!(
            select_gemini_auto_switch_target(&gemini_accounts),
            Some("pro-high".to_string())
        );
    }

    fn codex_account(
        id: &str,
        is_active: bool,
        five_hour_remaining_percent: Option<u8>,
        weekly_remaining_percent: Option<u8>,
    ) -> CodexAccountListItem {
        CodexAccountListItem {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            plan: Some("Plus".to_string()),
            account_id: Some(format!("acct-{id}")),
            is_active,
            last_authenticated_at: "0".to_string(),
            five_hour_remaining_percent,
            weekly_remaining_percent,
            five_hour_refresh_at: None,
            weekly_refresh_at: None,
            last_synced_at: Some("1775900000".to_string()),
            last_sync_error: None,
            credits_balance: None,
            needs_relogin: Some(false),
        }
    }

    fn claude_account(
        id: &str,
        is_active: bool,
        session_remaining_percent: Option<u8>,
        weekly_remaining_percent: Option<u8>,
    ) -> ClaudeAccountListItem {
        ClaudeAccountListItem {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            display_name: Some(id.to_string()),
            plan: Some("Pro".to_string()),
            account_hint: Some(format!("org-{id}")),
            is_active,
            last_authenticated_at: "0".to_string(),
            session_remaining_percent,
            session_refresh_at: None,
            weekly_remaining_percent,
            weekly_refresh_at: None,
            model_weekly_label: None,
            model_weekly_remaining_percent: None,
            model_weekly_refresh_at: None,
            last_synced_at: Some("1775900000".to_string()),
            last_sync_error: None,
            needs_relogin: Some(false),
        }
    }

    fn gemini_account(
        id: &str,
        is_active: bool,
        pro_remaining_percent: Option<u8>,
        flash_remaining_percent: Option<u8>,
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
            flash_remaining_percent,
            flash_lite_remaining_percent: None,
            pro_refresh_at: None,
            flash_refresh_at: None,
            flash_lite_refresh_at: None,
            last_synced_at: Some("1775900000".to_string()),
            last_sync_error: None,
            needs_relogin: Some(false),
        }
    }
}
