#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
pub fn shell_escape_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
pub fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
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
