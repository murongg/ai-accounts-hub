const FIVE_HOUR_WINDOW_SECS: u64 = 5 * 60 * 60;
const WEEKLY_WINDOW_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshWindows {
    pub five_hour_refresh_at: String,
    pub weekly_refresh_at: String,
}

pub fn next_refresh_at(base_secs: u64, window_secs: u64, now_secs: u64) -> u64 {
    if now_secs < base_secs {
        return base_secs + window_secs;
    }

    let elapsed = now_secs - base_secs;
    let windows_elapsed = (elapsed / window_secs) + 1;
    base_secs + (windows_elapsed * window_secs)
}

pub fn refresh_windows_from_last_authenticated_at(
    last_authenticated_at: &str,
    now_secs: u64,
) -> Option<RefreshWindows> {
    let base_secs = last_authenticated_at.parse::<u64>().ok()?;

    Some(RefreshWindows {
        five_hour_refresh_at: next_refresh_at(base_secs, FIVE_HOUR_WINDOW_SECS, now_secs)
            .to_string(),
        weekly_refresh_at: next_refresh_at(base_secs, WEEKLY_WINDOW_SECS, now_secs).to_string(),
    })
}

pub fn current_unix_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
