use ai_accounts_hub_lib::claude_usage::cli_probe::{
    parse_usage_output, ClaudeCliUsageProbeError,
};

#[test]
fn parses_session_weekly_and_opus_from_cli_output() {
    let usage = parse_usage_output(
        r#"
Current session
18% used
Resets in 3h

Current week
26% used
Resets in 6d 22h

Opus week
39% used
Resets in 6d 22h
"#,
    )
    .expect("usage");

    assert_eq!(
        usage.session.as_ref().map(|window| window.remaining_percent),
        Some(82)
    );
    assert_eq!(
        usage.weekly.as_ref().map(|window| window.remaining_percent),
        Some(74)
    );
    assert_eq!(usage.model_weekly_label.as_deref(), Some("Opus Weekly"));
    assert_eq!(
        usage.model_weekly.as_ref().map(|window| window.remaining_percent),
        Some(61)
    );
}

#[test]
fn parses_sonnet_when_opus_is_missing() {
    let usage = parse_usage_output(
        r#"
Current session
5% used
Resets in 1h 10m

Current week
40% used
Resets in 4d 3h

Sonnet week
11% used
Resets in 4d 3h
"#,
    )
    .expect("usage");

    assert_eq!(usage.model_weekly_label.as_deref(), Some("Sonnet Weekly"));
    assert_eq!(
        usage.model_weekly.as_ref().map(|window| window.remaining_percent),
        Some(89)
    );
}

#[test]
fn relogin_messages_are_classified() {
    let error = parse_usage_output("Authentication required. Please run claude login again.")
        .expect_err("should fail");

    assert!(matches!(error, ClaudeCliUsageProbeError::ReloginRequired(_)));
}
