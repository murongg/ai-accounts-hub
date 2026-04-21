use crate::claude_accounts::models::ClaudeAccountListItem;
use crate::codex_accounts::models::CodexAccountListItem;
use crate::gemini_accounts::models::GeminiAccountListItem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoSwitchThresholds {
    pub five_hour_percent: u8,
    pub weekly_percent: u8,
}

pub fn select_codex_auto_switch_target(accounts: &[CodexAccountListItem]) -> Option<String> {
    let active_account = accounts.iter().find(|account| account.is_active)?;
    let active_is_unusable = active_account.needs_relogin.unwrap_or(false)
        || active_account.five_hour_remaining_percent == Some(0)
        || active_account.weekly_remaining_percent == Some(0);

    if !active_is_unusable {
        return None;
    }

    accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| !account.is_active && !account.needs_relogin.unwrap_or(false))
        .filter_map(|(index, account)| {
            let weekly = account.weekly_remaining_percent?;
            let five_hour = account.five_hour_remaining_percent?;

            (weekly > 0 && five_hour > 0)
                .then(|| (index, weekly, five_hour, account.id.to_string()))
        })
        .max_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| right.0.cmp(&left.0))
        })
        .map(|(_, _, _, account_id)| account_id)
}

pub fn select_codex_auto_switch_target_with_thresholds(
    accounts: &[CodexAccountListItem],
    thresholds: AutoSwitchThresholds,
) -> Option<String> {
    let active_account = accounts.iter().find(|account| account.is_active)?;
    let active_is_unusable = active_account.needs_relogin.unwrap_or(false)
        || quota_reached_threshold(
            active_account.five_hour_remaining_percent,
            thresholds.five_hour_percent,
        )
        || quota_reached_threshold(
            active_account.weekly_remaining_percent,
            thresholds.weekly_percent,
        );

    if !active_is_unusable {
        return None;
    }

    select_codex_switch_candidate_above_thresholds(accounts, thresholds)
}

pub fn select_codex_switch_candidate_above_thresholds(
    accounts: &[CodexAccountListItem],
    thresholds: AutoSwitchThresholds,
) -> Option<String> {
    accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| !account.is_active && !account.needs_relogin.unwrap_or(false))
        .filter_map(|(index, account)| {
            let weekly = account.weekly_remaining_percent?;
            let five_hour = account.five_hour_remaining_percent?;

            (weekly > thresholds.weekly_percent && five_hour > thresholds.five_hour_percent)
                .then(|| (index, weekly, five_hour, account.id.to_string()))
        })
        .max_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| right.0.cmp(&left.0))
        })
        .map(|(_, _, _, account_id)| account_id)
}

pub fn select_claude_auto_switch_target(accounts: &[ClaudeAccountListItem]) -> Option<String> {
    let active_account = accounts.iter().find(|account| account.is_active)?;
    let active_is_unusable = active_account.needs_relogin.unwrap_or(false)
        || active_account.session_remaining_percent == Some(0)
        || active_account.weekly_remaining_percent == Some(0);

    if !active_is_unusable {
        return None;
    }

    accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| !account.is_active && !account.needs_relogin.unwrap_or(false))
        .filter_map(|(index, account)| {
            let weekly = account.weekly_remaining_percent?;
            let session = account.session_remaining_percent?;

            (weekly > 0 && session > 0).then(|| {
                (
                    index,
                    weekly,
                    session,
                    account.model_weekly_remaining_percent.unwrap_or(0),
                    account.id.to_string(),
                )
            })
        })
        .max_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| left.3.cmp(&right.3))
                .then_with(|| right.0.cmp(&left.0))
        })
        .map(|(_, _, _, _, account_id)| account_id)
}

pub fn select_claude_auto_switch_target_with_thresholds(
    accounts: &[ClaudeAccountListItem],
    _thresholds: AutoSwitchThresholds,
) -> Option<String> {
    select_claude_auto_switch_target(accounts)
}

pub fn select_gemini_auto_switch_target(accounts: &[GeminiAccountListItem]) -> Option<String> {
    let active_account = accounts.iter().find(|account| account.is_active)?;
    let active_is_unusable = active_account.needs_relogin.unwrap_or(false)
        || matches!(
            (
                active_account.pro_remaining_percent,
                active_account.flash_remaining_percent,
                active_account.flash_lite_remaining_percent,
            ),
            (Some(0), Some(0), Some(0))
        );

    if !active_is_unusable {
        return None;
    }

    accounts
        .iter()
        .enumerate()
        .filter(|(_, account)| !account.is_active && !account.needs_relogin.unwrap_or(false))
        .filter_map(|(index, account)| {
            let pro = account.pro_remaining_percent.unwrap_or(0);
            let flash = account.flash_remaining_percent.unwrap_or(0);
            let flash_lite = account.flash_lite_remaining_percent.unwrap_or(0);

            (pro > 0 || flash > 0 || flash_lite > 0)
                .then(|| (index, pro, flash, flash_lite, account.id.to_string()))
        })
        .max_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| left.3.cmp(&right.3))
                .then_with(|| right.0.cmp(&left.0))
        })
        .map(|(_, _, _, _, account_id)| account_id)
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

fn quota_reached_threshold(quota_percent: Option<u8>, threshold_percent: u8) -> bool {
    quota_percent.is_some_and(|quota| quota <= threshold_percent)
}

#[cfg(test)]
mod tests {
    use super::{
        select_auto_switch_target, select_claude_auto_switch_target,
        select_claude_auto_switch_target_with_thresholds, select_codex_auto_switch_target,
        select_codex_auto_switch_target_with_thresholds, select_codex_switch_candidate_above_thresholds,
        select_gemini_auto_switch_target, AutoSwitchThresholds,
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
    fn codex_auto_switch_prioritizes_weekly_quota_before_five_hour_quota() {
        let accounts = vec![
            codex_account("active", true, Some(0), Some(0)),
            codex_account("weekly-high", false, Some(12), Some(99)),
            codex_account("five-hour-high", false, Some(91), Some(1)),
        ];

        assert_eq!(
            select_codex_auto_switch_target(&accounts),
            Some("weekly-high".to_string())
        );
    }

    #[test]
    fn codex_auto_switch_triggers_when_weekly_quota_is_depleted() {
        let accounts = vec![
            codex_account("active", true, Some(80), Some(0)),
            codex_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_codex_auto_switch_target(&accounts),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn codex_auto_switch_triggers_when_five_hour_threshold_is_reached() {
        let accounts = vec![
            codex_account("active", true, Some(9), Some(80)),
            codex_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_codex_auto_switch_target_with_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 10,
                    weekly_percent: 0,
                },
            ),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn codex_auto_switch_ignores_candidates_missing_weekly_or_five_hour_quota() {
        let accounts = vec![
            codex_account("active", true, Some(0), Some(0)),
            codex_account("missing-weekly", false, Some(90), None),
            codex_account("missing-five-hour", false, None, Some(90)),
            codex_account("candidate", false, Some(40), Some(35)),
        ];

        assert_eq!(
            select_codex_auto_switch_target(&accounts),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn claude_auto_switch_prioritizes_weekly_before_session() {
        let claude_accounts = vec![
            claude_account("active", true, Some(0), Some(0)),
            claude_account("weekly-high", false, Some(18), Some(99)),
            claude_account("session-high", false, Some(72), Some(10)),
        ];

        assert_eq!(
            select_claude_auto_switch_target(&claude_accounts),
            Some("weekly-high".to_string())
        );
    }

    #[test]
    fn claude_auto_switch_triggers_when_weekly_quota_is_depleted() {
        let accounts = vec![
            claude_account("active", true, Some(80), Some(0)),
            claude_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_claude_auto_switch_target(&accounts),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn claude_auto_switch_does_not_trigger_before_quota_is_depleted_even_with_threshold_configured() {
        let accounts = vec![
            claude_account("active", true, Some(80), Some(4)),
            claude_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_claude_auto_switch_target_with_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 0,
                    weekly_percent: 5,
                },
            ),
            None
        );
    }

    #[test]
    fn claude_auto_switch_still_triggers_when_session_quota_is_depleted() {
        let accounts = vec![
            claude_account("active", true, Some(0), Some(80)),
            claude_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_claude_auto_switch_target_with_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 0,
                    weekly_percent: 5,
                },
            ),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn codex_auto_switch_requires_candidate_quotas_to_stay_above_thresholds() {
        let accounts = vec![
            codex_account("active", true, Some(10), Some(80)),
            codex_account("five-hour-at-threshold", false, Some(10), Some(90)),
            codex_account("weekly-at-threshold", false, Some(90), Some(5)),
            codex_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_codex_auto_switch_target_with_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 10,
                    weekly_percent: 5,
                },
            ),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn codex_auto_switch_skips_switch_when_no_candidate_stays_above_thresholds() {
        let accounts = vec![
            codex_account("active", true, Some(10), Some(80)),
            codex_account("five-hour-at-threshold", false, Some(10), Some(90)),
            codex_account("weekly-at-threshold", false, Some(90), Some(5)),
            codex_account("both-below", false, Some(9), Some(4)),
        ];

        assert_eq!(
            select_codex_auto_switch_target_with_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 10,
                    weekly_percent: 5,
                },
            ),
            None
        );
    }

    #[test]
    fn claude_auto_switch_ignores_candidates_missing_session_or_weekly_quota() {
        let accounts = vec![
            claude_account("active", true, Some(0), Some(0)),
            claude_account("missing-weekly", false, Some(90), None),
            claude_account("missing-session", false, None, Some(90)),
            claude_account("candidate", false, Some(40), Some(35)),
        ];

        assert_eq!(
            select_claude_auto_switch_target(&accounts),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn gemini_auto_switch_waits_until_all_quota_buckets_are_depleted() {
        let still_usable_accounts = vec![
            gemini_account("active", true, Some(0), Some(18), Some(0)),
            gemini_account("candidate", false, Some(10), Some(0), Some(0)),
        ];

        let exhausted_accounts = vec![
            gemini_account("active", true, Some(0), Some(0), Some(0)),
            gemini_account("candidate", false, Some(10), Some(0), Some(0)),
        ];

        assert_eq!(
            select_gemini_auto_switch_target(&still_usable_accounts),
            None
        );
        assert_eq!(
            select_gemini_auto_switch_target(&exhausted_accounts),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn gemini_auto_switch_uses_pro_flash_and_flash_lite_as_ranked_tie_breakers() {
        let accounts = vec![
            gemini_account("active", true, Some(0), Some(0), Some(0)),
            gemini_account("flash-high", false, Some(10), Some(99), Some(10)),
            gemini_account("pro-high", false, Some(88), Some(10), Some(10)),
        ];

        assert_eq!(
            select_gemini_auto_switch_target(&accounts),
            Some("pro-high".to_string())
        );
    }

    #[test]
    fn codex_auto_switch_does_not_trigger_when_threshold_is_not_reached() {
        let accounts = vec![
            codex_account("active", true, Some(11), Some(80)),
            codex_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_codex_auto_switch_target_with_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 10,
                    weekly_percent: 10,
                },
            ),
            None
        );
    }

    #[test]
    fn forced_codex_switch_selects_best_candidate_above_thresholds() {
        let accounts = vec![
            codex_account("active", true, Some(14), Some(80)),
            codex_account("five-hour-below-threshold", false, Some(10), Some(99)),
            codex_account("weekly-below-threshold", false, Some(90), Some(5)),
            codex_account("candidate", false, Some(55), Some(72)),
        ];

        assert_eq!(
            select_codex_switch_candidate_above_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 10,
                    weekly_percent: 5,
                },
            ),
            Some("candidate".to_string())
        );
    }

    #[test]
    fn forced_codex_switch_skips_when_no_candidate_stays_above_thresholds() {
        let accounts = vec![
            codex_account("active", true, Some(14), Some(80)),
            codex_account("five-hour-at-threshold", false, Some(10), Some(99)),
            codex_account("weekly-at-threshold", false, Some(90), Some(5)),
        ];

        assert_eq!(
            select_codex_switch_candidate_above_thresholds(
                &accounts,
                AutoSwitchThresholds {
                    five_hour_percent: 10,
                    weekly_percent: 5,
                },
            ),
            None
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
            label: None,
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
            refresh_accelerated_until: None,
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
            label: None,
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
        flash_lite_remaining_percent: Option<u8>,
    ) -> GeminiAccountListItem {
        GeminiAccountListItem {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            label: None,
            subject: Some(format!("subject-{id}")),
            auth_type: Some("oauth-personal".to_string()),
            plan: Some("Pro".to_string()),
            is_active,
            last_authenticated_at: "0".to_string(),
            pro_remaining_percent,
            flash_remaining_percent,
            flash_lite_remaining_percent,
            pro_refresh_at: None,
            flash_refresh_at: None,
            flash_lite_refresh_at: None,
            last_synced_at: Some("1775900000".to_string()),
            last_sync_error: None,
            needs_relogin: Some(false),
        }
    }
}
