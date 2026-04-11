use std::fs;
use std::path::Path;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid file path".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("failed to create parent dir: {error}"))?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid file name".to_string())?;
    let temp_path = parent.join(format!(".{file_name}.tmp"));

    fs::write(&temp_path, bytes).map_err(|error| format!("failed to write temp file: {error}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("failed to remove old file: {error}"))?;
    }
    fs::rename(&temp_path, path).map_err(|error| format!("failed to replace file: {error}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn atomic_write_creates_missing_parent_directories() {
        let root = temp_test_dir("atomic-write-create");
        let path = root.join("nested/dir/config.json");

        atomic_write(&path, br#"{"ok":true}"#).expect("write file");

        assert_eq!(
            fs::read_to_string(path).expect("read file"),
            r#"{"ok":true}"#
        );
    }

    #[test]
    fn atomic_write_replaces_existing_file_contents() {
        let root = temp_test_dir("atomic-write-replace");
        let path = root.join("state.json");
        fs::create_dir_all(&root).expect("root dir");
        fs::write(&path, "old").expect("seed file");

        atomic_write(&path, b"new").expect("replace file");

        assert_eq!(fs::read_to_string(path).expect("read file"), "new");
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aihub-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir");
        path
    }
}
