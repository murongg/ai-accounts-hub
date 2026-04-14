pub fn timestamp_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
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
}
