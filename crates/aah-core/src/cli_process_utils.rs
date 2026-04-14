#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
static UNIQUE_SUFFIX_COUNTER: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "macos")]
pub fn shell_escape_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
pub fn unique_suffix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let sequence = UNIQUE_SUFFIX_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{timestamp}-{sequence}")
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn shell_escape_path_wraps_in_single_quotes_and_escapes_embedded_quotes() {
        let escaped = shell_escape_path(Path::new("/tmp/with ' quote"));
        assert_eq!(escaped, "'/tmp/with '\\'' quote'");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn unique_suffix_changes_across_calls() {
        let first = unique_suffix();
        let second = unique_suffix();

        assert_ne!(first, second);
    }
}
