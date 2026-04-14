use std::path::PathBuf;

use crate::managed_root::{
    migrate_legacy_root, normalize_managed_root_metadata, resolve_managed_root, ManagedRoot,
};
use crate::startup_account_import::import_logged_in_accounts;

#[derive(Debug, Clone)]
pub struct BootstrapContext {
    pub managed_root: PathBuf,
    pub user_home: PathBuf,
    pub import_warnings: Vec<String>,
}

pub fn bootstrap_managed_root(
    home_override: Option<PathBuf>,
    data_dir_override: Option<PathBuf>,
) -> Result<ManagedRoot, String> {
    let user_home = match home_override {
        Some(home) => home,
        None => {
            dirs::home_dir().ok_or_else(|| "failed to resolve user home directory".to_string())?
        }
    };
    let managed = resolve_managed_root(&user_home, data_dir_override);
    migrate_legacy_root(&managed)?;
    normalize_managed_root_metadata(&managed)?;
    Ok(managed)
}

pub fn bootstrap_context(
    home_override: Option<PathBuf>,
    data_dir_override: Option<PathBuf>,
) -> Result<BootstrapContext, String> {
    let managed = bootstrap_managed_root(home_override, data_dir_override)?;
    let import_outcome = import_logged_in_accounts(managed.root.clone(), managed.user_home.clone());
    Ok(BootstrapContext {
        managed_root: managed.root,
        user_home: managed.user_home,
        import_warnings: import_outcome.errors,
    })
}
