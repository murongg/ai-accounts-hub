use ai_accounts_hub_lib::codex_accounts::schedule::{
    next_refresh_at, refresh_windows_from_last_authenticated_at,
};

#[test]
fn next_refresh_at_returns_the_next_future_boundary() {
    let base = 1_700_000_000_u64;
    let window = 5 * 60 * 60;

    assert_eq!(next_refresh_at(base, window, base), base + window);
    assert_eq!(next_refresh_at(base, window, base + 60), base + window);
    assert_eq!(
        next_refresh_at(base, window, base + window),
        base + (window * 2)
    );
}

#[test]
fn refresh_windows_roll_forward_when_the_base_is_old() {
    let base = 1_700_000_000_u64;
    let now = base + (9 * 24 * 60 * 60);

    let windows = refresh_windows_from_last_authenticated_at(&base.to_string(), now)
        .expect("refresh windows");

    let five_hour = windows.five_hour_refresh_at.parse::<u64>().expect("5h ts");
    let weekly = windows.weekly_refresh_at.parse::<u64>().expect("weekly ts");

    assert!(five_hour > now);
    assert!(weekly > now);
}
