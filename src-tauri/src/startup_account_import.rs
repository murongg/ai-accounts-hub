use std::path::PathBuf;

use crate::claude_accounts::paths::ClaudeAccountPaths;
use crate::claude_accounts::service::ClaudeAccountService;
use crate::codex_accounts::paths::CodexAccountPaths;
use crate::codex_accounts::service::CodexAccountService;
use crate::gemini_accounts::paths::GeminiAccountPaths;
use crate::gemini_accounts::service::GeminiAccountService;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct StartupAccountImportOutcome {
    pub imported_count: usize,
    pub errors: Vec<String>,
}

impl StartupAccountImportOutcome {
    pub fn imported_any(&self) -> bool {
        self.imported_count > 0
    }
}

pub fn import_logged_in_accounts(
    app_data_dir: PathBuf,
    user_home: PathBuf,
) -> StartupAccountImportOutcome {
    let codex_paths = CodexAccountPaths::from_roots(app_data_dir.clone(), user_home.clone());
    let claude_paths = ClaudeAccountPaths::from_roots(app_data_dir.clone(), user_home.clone());
    let gemini_paths = GeminiAccountPaths::from_roots(app_data_dir, user_home);

    run_import_actions(
        || {
            CodexAccountService::with_process_runner(codex_paths.clone())
                .import_current_account_if_missing()
                .map(|account| account.is_some())
        },
        || {
            let mut service = ClaudeAccountService::with_process_runner(claude_paths.clone());
            service
                .import_current_account_if_missing()
                .map(|account| account.is_some())
        },
        || {
            GeminiAccountService::with_process_runner(gemini_paths.clone())
                .import_current_account_if_missing()
                .map(|account| account.is_some())
        },
    )
}

fn run_import_actions<F1, F2, F3>(
    mut import_codex: F1,
    mut import_claude: F2,
    mut import_gemini: F3,
) -> StartupAccountImportOutcome
where
    F1: FnMut() -> Result<bool, String>,
    F2: FnMut() -> Result<bool, String>,
    F3: FnMut() -> Result<bool, String>,
{
    let mut outcome = StartupAccountImportOutcome::default();

    apply_import_result("Codex", import_codex(), &mut outcome);
    apply_import_result("Claude", import_claude(), &mut outcome);
    apply_import_result("Gemini", import_gemini(), &mut outcome);

    outcome
}

fn apply_import_result(
    provider: &str,
    result: Result<bool, String>,
    outcome: &mut StartupAccountImportOutcome,
) {
    match result {
        Ok(true) => outcome.imported_count += 1,
        Ok(false) => {}
        Err(error) => outcome
            .errors
            .push(format!("{provider} account auto-import failed: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{run_import_actions, StartupAccountImportOutcome};

    #[test]
    fn run_import_actions_counts_successful_imports() {
        let outcome = run_import_actions(|| Ok(true), || Ok(false), || Ok(true));

        assert_eq!(
            outcome,
            StartupAccountImportOutcome {
                imported_count: 2,
                errors: Vec::new(),
            }
        );
        assert!(outcome.imported_any());
    }

    #[test]
    fn run_import_actions_continues_when_a_provider_fails() {
        let outcome = run_import_actions(|| Err("broken".to_string()), || Ok(true), || Ok(false));

        assert_eq!(outcome.imported_count, 1);
        assert_eq!(
            outcome.errors,
            vec!["Codex account auto-import failed: broken".to_string()]
        );
    }
}
