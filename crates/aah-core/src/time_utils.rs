use std::time::{SystemTime, UNIX_EPOCH};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn timestamp_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub fn format_refresh_countdown(raw: Option<&str>) -> String {
    format_refresh_countdown_at(raw, SystemTime::now())
}

pub fn format_refresh_countdown_at(raw: Option<&str>, now: SystemTime) -> String {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return "--:--".to_string();
    };
    let Some(refresh_at) = parse_refresh_at_seconds(raw) else {
        return raw.to_string();
    };
    let now_seconds = now
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    if refresh_at <= now_seconds {
        return "soon".to_string();
    }

    format_countdown_seconds(refresh_at - now_seconds)
}

fn parse_refresh_at_seconds(raw: &str) -> Option<u64> {
    raw.parse::<u64>().ok().or_else(|| {
        OffsetDateTime::parse(raw, &Rfc3339)
            .ok()
            .and_then(|timestamp| u64::try_from(timestamp.unix_timestamp()).ok())
    })
}

fn format_countdown_seconds(seconds: u64) -> String {
    let total_minutes = seconds / 60;
    if total_minutes < 1 {
        return "soon".to_string();
    }
    if total_minutes < 60 {
        return format!("{total_minutes}m");
    }

    let total_hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if total_hours < 24 {
        return if minutes > 0 {
            format!("{total_hours}h {minutes}m")
        } else {
            format!("{total_hours}h")
        };
    }

    let days = total_hours / 24;
    let hours = total_hours % 24;
    if hours > 0 {
        format!("{days}d {hours}h")
    } else {
        format!("{days}d")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn timestamp_string_returns_unix_seconds_as_digits() {
        let timestamp = timestamp_string();

        assert!(timestamp.parse::<u64>().is_ok());
    }

    #[test]
    fn timestamp_string_falls_within_current_second_range() {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs();

        let timestamp = timestamp_string()
            .parse::<u64>()
            .expect("numeric timestamp");

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_secs();

        assert!(timestamp >= before);
        assert!(timestamp <= after);
    }

    #[test]
    fn format_refresh_countdown_supports_unix_seconds() {
        let now = UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);

        assert_eq!(
            format_refresh_countdown_at(Some("1700003900"), now),
            "1h 5m"
        );
    }

    #[test]
    fn format_refresh_countdown_supports_rfc3339() {
        let now = UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);

        assert_eq!(
            format_refresh_countdown_at(Some("2023-11-14T23:18:20Z"), now),
            "1h 5m"
        );
    }
}
