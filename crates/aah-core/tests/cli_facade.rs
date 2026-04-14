use aah_core::bootstrap::bootstrap_context;
use aah_core::cli_facade::{CliFacade, Provider, SwitchSelection};
use std::fs;
use std::path::PathBuf;

fn temp_home(prefix: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("aah-cli-facade-{}-{}", prefix, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir");
    path
}

#[test]
fn list_current_and_refresh_run_against_the_bootstrapped_root() {
    let home = temp_home("list-current-refresh");
    let context = bootstrap_context(Some(home), None).expect("bootstrap context");
    let facade = CliFacade::new(context);

    let list = facade.list(None).expect("list");
    let current = facade.current(None).expect("current");
    let refresh = facade.refresh(None).expect("refresh");

    assert!(list.is_empty());
    assert_eq!(
        current.iter().map(|row| row.provider).collect::<Vec<_>>(),
        vec![Provider::Codex, Provider::Claude, Provider::Gemini]
    );
    assert_eq!(refresh.len(), 3);
}

#[test]
fn switch_requires_an_explicit_provider_and_selector() {
    let home = temp_home("switch");
    let context = bootstrap_context(Some(home), None).expect("bootstrap context");
    let facade = CliFacade::new(context);

    let error = facade
        .switch(
            Provider::Codex,
            SwitchSelection::Email("missing@example.com".to_string()),
        )
        .expect_err("missing account should fail");

    assert!(error.to_string().contains("missing@example.com"));
}
