use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedRoot {
    pub root: PathBuf,
    pub user_home: PathBuf,
}

pub fn legacy_root_candidates(home: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "macos")]
    roots.push(
        home.join("Library")
            .join("Application Support")
            .join("com.murong.ai-accounts-hub"),
    );

    #[cfg(target_os = "linux")]
    roots.push(
        home.join(".local")
            .join("share")
            .join("com.murong.ai-accounts-hub"),
    );

    #[cfg(target_os = "windows")]
    roots.push(
        home.join("AppData")
            .join("Roaming")
            .join("com.murong.ai-accounts-hub"),
    );

    roots
}

pub fn resolve_managed_root(home: &Path, override_dir: Option<PathBuf>) -> ManagedRoot {
    ManagedRoot {
        root: override_dir.unwrap_or_else(|| home.join(".ai-accounts-hub")),
        user_home: home.to_path_buf(),
    }
}

pub fn migrate_legacy_root(root: &ManagedRoot) -> Result<(), String> {
    if root.root.exists() {
        return Ok(());
    }

    for legacy in legacy_root_candidates(&root.user_home) {
        if !legacy.exists() {
            continue;
        }

        let parent = root
            .root
            .parent()
            .ok_or_else(|| format!("managed root has no parent: {}", root.root.display()))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create managed root parent: {error}"))?;
        fs::rename(&legacy, &root.root).map_err(|error| {
            format!(
                "failed to migrate legacy data root {}: {error}",
                legacy.display()
            )
        })?;
        return Ok(());
    }

    fs::create_dir_all(&root.root).map_err(|error| {
        format!(
            "failed to create managed root {}: {error}",
            root.root.display()
        )
    })?;
    Ok(())
}

pub fn normalize_managed_root_metadata(root: &ManagedRoot) -> Result<(), String> {
    let legacy_roots = legacy_root_candidates(&root.user_home);
    normalize_account_index_paths(
        &root.root.join("codex").join("accounts.json"),
        &legacy_roots,
        &root.root,
    )?;
    normalize_account_index_paths(
        &root.root.join("gemini").join("accounts.json"),
        &legacy_roots,
        &root.root,
    )?;
    Ok(())
}

fn normalize_account_index_paths(
    index_path: &Path,
    legacy_roots: &[PathBuf],
    managed_root: &Path,
) -> Result<(), String> {
    let text = match fs::read_to_string(index_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "failed to read account index {}: {error}",
                index_path.display()
            ))
        }
    };

    let mut index: Value = serde_json::from_str(&text).map_err(|error| {
        format!(
            "failed to parse account index {}: {error}",
            index_path.display()
        )
    })?;
    let Some(accounts) = index.get_mut("accounts").and_then(Value::as_array_mut) else {
        return Ok(());
    };

    let mut changed = false;
    for account in accounts {
        let Some(object) = account.as_object_mut() else {
            continue;
        };
        let Some(path_value) = object.get_mut("managed_home_path") else {
            continue;
        };
        let Some(path_str) = path_value.as_str() else {
            continue;
        };
        let Some(normalized_path) =
            normalize_managed_home_path(path_str, legacy_roots, managed_root)
        else {
            continue;
        };

        *path_value = Value::String(normalized_path.display().to_string());
        changed = true;
    }

    if changed {
        let bytes = serde_json::to_vec_pretty(&index).map_err(|error| {
            format!(
                "failed to serialize account index {}: {error}",
                index_path.display()
            )
        })?;
        fs::write(index_path, bytes).map_err(|error| {
            format!(
                "failed to write account index {}: {error}",
                index_path.display()
            )
        })?;
    }

    Ok(())
}

fn normalize_managed_home_path(
    path: &str,
    legacy_roots: &[PathBuf],
    managed_root: &Path,
) -> Option<PathBuf> {
    let stored_path = Path::new(path);
    legacy_roots.iter().find_map(|legacy_root| {
        stored_path
            .strip_prefix(legacy_root)
            .ok()
            .map(|relative| managed_root.join(relative))
    })
}
