use aah_core::claude_accounts::paths::ClaudeAccountPaths;
use aah_core::codex_accounts::paths::CodexAccountPaths;
use aah_core::gemini_accounts::paths::GeminiAccountPaths;

#[test]
fn provider_paths_are_available_from_aah_core() {
    let root = std::env::temp_dir().join("aah-core-shared-services");
    let home = root.join("home");

    let codex = CodexAccountPaths::for_test(root.clone(), home.clone());
    let claude = ClaudeAccountPaths::for_test(root.clone(), home.clone());
    let gemini = GeminiAccountPaths::for_test(root, home);

    assert!(codex.account_index_path.ends_with("accounts.json"));
    assert!(claude.metadata_index_path.ends_with("accounts.json"));
    assert!(gemini.account_index_path.ends_with("accounts.json"));
}
