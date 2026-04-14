use std::path::PathBuf;

use crate::managed_root::{migrate_legacy_root, resolve_managed_root, ManagedRoot};

pub fn bootstrap_managed_root(
    home_override: Option<PathBuf>,
    data_dir_override: Option<PathBuf>,
) -> Result<ManagedRoot, String> {
    let user_home = match home_override {
        Some(home) => home,
        None => dirs::home_dir()
            .ok_or_else(|| "failed to resolve user home directory".to_string())?,
    };
    let managed = resolve_managed_root(&user_home, data_dir_override);
    migrate_legacy_root(&managed)?;
    Ok(managed)
}
