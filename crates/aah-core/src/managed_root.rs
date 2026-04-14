use std::fs;
use std::path::{Path, PathBuf};

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
