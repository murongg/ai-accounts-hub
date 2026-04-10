#[cfg(target_os = "macos")]
use std::ffi::CString;

#[cfg(target_os = "macos")]
use std::os::raw::c_int;

#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "QuartzCore", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "SwiftUI", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "aah_status_bar_bridge", kind = "static")]
unsafe extern "C" {
    fn aah_status_bar_bridge_debug_visible_tab_count() -> c_int;
    fn aah_status_bar_bridge_debug_active_section_index_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_total_metric_count_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_footer_action_count() -> c_int;
    fn aah_status_bar_bridge_debug_selected_tab_after_action_from_json(
        payload_json: *const i8,
        action_json: *const i8,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_action_keeps_menu_open(action_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_action_is_handled_locally(action_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_shows_account_chips_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_visible_detail_section_count_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_pinned_detail_section_count_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_scrollable_detail_section_count_from_json(
        payload_json: *const i8,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_account_chip_layout_axis_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_hosting_view_allows_vibrancy() -> c_int;
    fn aah_status_bar_bridge_debug_hosting_view_accepts_first_mouse() -> c_int;
    fn aah_status_bar_bridge_debug_uses_outer_panel_chrome() -> c_int;
    fn aah_status_bar_bridge_debug_uses_scrollable_detail_container() -> c_int;
    fn aah_status_bar_bridge_debug_menu_refreshes_layout_on_open() -> c_int;
    fn aah_status_bar_bridge_debug_panel_width() -> c_int;
    fn aah_status_bar_bridge_debug_switcher_height() -> c_int;
    fn aah_status_bar_bridge_debug_hover_corner_radius() -> c_int;
    fn aah_status_bar_bridge_debug_footer_action_row_height() -> c_int;
    fn aah_status_bar_bridge_debug_footer_icon_column_width() -> c_int;
    fn aah_status_bar_bridge_debug_footer_chevron_column_width() -> c_int;
    fn aah_status_bar_bridge_debug_footer_action_horizontal_padding() -> c_int;
    fn aah_status_bar_bridge_debug_footer_action_vertical_padding() -> c_int;
    fn aah_status_bar_bridge_debug_panel_content_padding() -> c_int;
    fn aah_status_bar_bridge_debug_interactive_row_horizontal_padding() -> c_int;
    fn aah_status_bar_bridge_debug_interactive_row_vertical_padding() -> c_int;
    fn aah_status_bar_bridge_debug_account_section_hover_horizontal_padding() -> c_int;
    fn aah_status_bar_bridge_debug_account_section_hover_vertical_padding() -> c_int;
    fn aah_status_bar_bridge_debug_quota_tone_for_percent(percent: u8) -> c_int;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_presentation_keeps_tabs_actions_and_active_section_order() {
    let payload = CString::new(
        r#"{"selectedTab":"codex","sections":[{"id":"codex:inactive","providerId":"codex","providerTitle":"Codex","email":"inactive@example.com","subtitle":"Updated 8m ago","plan":"Pro","isActive":false,"needsRelogin":false,"metrics":[{"title":"Messages","percent":20,"leftText":"80% left","resetText":"Resets in 3h"}],"switchAccountId":"acct-inactive"},{"id":"codex:active","providerId":"codex","providerTitle":"Codex","email":"active@example.com","subtitle":"Updated 1m ago","plan":"Pro","isActive":true,"needsRelogin":false,"metrics":[{"title":"Messages","percent":70,"leftText":"30% left","resetText":"Resets in 1h"},{"title":"Requests","percent":45,"leftText":"55% left","resetText":"Resets tomorrow"}],"switchAccountId":null}]}"#,
    )
    .unwrap();

    let tab_count = unsafe { aah_status_bar_bridge_debug_visible_tab_count() };
    let active_index =
        unsafe { aah_status_bar_bridge_debug_active_section_index_from_json(payload.as_ptr()) };
    let metric_count = unsafe { aah_status_bar_bridge_debug_total_metric_count_from_json(payload.as_ptr()) };
    let footer_count = unsafe { aah_status_bar_bridge_debug_footer_action_count() };

    assert_eq!(tab_count, 3);
    assert_eq!(active_index, 0);
    assert_eq!(metric_count, 3);
    assert_eq!(footer_count, 3);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_select_tab_action_updates_visible_tab_immediately() {
    let payload = CString::new(
        r#"{"selectedTab":"overview","sections":[{"id":"codex:active","providerId":"codex","providerTitle":"Codex","email":"active@example.com","subtitle":"Updated just now","plan":"Pro","isActive":true,"needsRelogin":false,"metrics":[{"title":"Session","percent":70,"leftText":"30% left","resetText":"Resets in 1h"}],"switchAccountId":null}]}"#,
    )
    .unwrap();
    let action = CString::new(r#"{"type":"select_tab","tab":"gemini"}"#).unwrap();

    let selected_tab =
        unsafe { aah_status_bar_bridge_debug_selected_tab_after_action_from_json(payload.as_ptr(), action.as_ptr()) };

    assert_eq!(selected_tab, 3);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_select_tab_action_keeps_the_menu_open() {
    let action = CString::new(r#"{"type":"select_tab","tab":"gemini"}"#).unwrap();

    let keeps_menu_open =
        unsafe { aah_status_bar_bridge_debug_action_keeps_menu_open(action.as_ptr()) };

    assert_eq!(keeps_menu_open, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_quit_action_is_handled_locally_after_dismissing_the_menu() {
    let action = CString::new(r#"{"type":"quit"}"#).unwrap();

    let keeps_menu_open =
        unsafe { aah_status_bar_bridge_debug_action_keeps_menu_open(action.as_ptr()) };
    let handled_locally =
        unsafe { aah_status_bar_bridge_debug_action_is_handled_locally(action.as_ptr()) };

    assert_eq!(keeps_menu_open, 0);
    assert_eq!(handled_locally, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_stacks_provider_accounts_as_vertical_detail_sections() {
    let payload = CString::new(
        r#"{"selectedTab":"codex","sections":[{"id":"codex:active","providerId":"codex","providerTitle":"Codex","email":"active@example.com","subtitle":"Updated just now","plan":"Pro","isActive":true,"needsRelogin":false,"metrics":[],"switchAccountId":null},{"id":"codex:other","providerId":"codex","providerTitle":"Codex","email":"other@example.com","subtitle":"Updated 5m ago","plan":"Free","isActive":false,"needsRelogin":false,"metrics":[],"switchAccountId":"acct-other"}]}"#,
    )
    .unwrap();

    let shows_account_chips =
        unsafe { aah_status_bar_bridge_debug_shows_account_chips_from_json(payload.as_ptr()) };
    let visible_detail_sections =
        unsafe { aah_status_bar_bridge_debug_visible_detail_section_count_from_json(payload.as_ptr()) };
    let axis = unsafe { aah_status_bar_bridge_debug_account_chip_layout_axis_from_json(payload.as_ptr()) };

    assert_eq!(shows_account_chips, 0);
    assert_eq!(visible_detail_sections, 2);
    assert_eq!(axis, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_keeps_current_account_pinned_and_scrolls_remaining_accounts() {
    let payload = CString::new(
        r#"{"selectedTab":"codex","sections":[{"id":"codex:active","providerId":"codex","providerTitle":"Codex","email":"active@example.com","subtitle":"Updated just now","plan":"Pro","isActive":true,"needsRelogin":false,"metrics":[],"switchAccountId":null},{"id":"codex:other-1","providerId":"codex","providerTitle":"Codex","email":"other-1@example.com","subtitle":"Updated 5m ago","plan":"Free","isActive":false,"needsRelogin":false,"metrics":[],"switchAccountId":"acct-other-1"},{"id":"codex:other-2","providerId":"codex","providerTitle":"Codex","email":"other-2@example.com","subtitle":"Updated 12m ago","plan":"Free","isActive":false,"needsRelogin":false,"metrics":[],"switchAccountId":"acct-other-2"}]}"#,
    )
    .unwrap();

    let pinned_sections =
        unsafe { aah_status_bar_bridge_debug_pinned_detail_section_count_from_json(payload.as_ptr()) };
    let scrollable_sections =
        unsafe { aah_status_bar_bridge_debug_scrollable_detail_section_count_from_json(payload.as_ptr()) };
    let visible_detail_sections =
        unsafe { aah_status_bar_bridge_debug_visible_detail_section_count_from_json(payload.as_ptr()) };

    assert_eq!(pinned_sections, 1);
    assert_eq!(scrollable_sections, 2);
    assert_eq!(visible_detail_sections, 3);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_hosting_view_enables_menu_vibrancy() {
    let allows_vibrancy = unsafe { aah_status_bar_bridge_debug_hosting_view_allows_vibrancy() };

    assert_eq!(allows_vibrancy, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_hosting_view_accepts_first_mouse_for_immediate_menu_clicks() {
    let accepts_first_mouse = unsafe { aah_status_bar_bridge_debug_hosting_view_accepts_first_mouse() };

    assert_eq!(accepts_first_mouse, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_relies_on_menu_background_instead_of_custom_outer_panel() {
    let uses_outer_panel_chrome = unsafe { aah_status_bar_bridge_debug_uses_outer_panel_chrome() };

    assert_eq!(uses_outer_panel_chrome, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_uses_scrollable_detail_container_for_account_sections() {
    let uses_scrollable_detail_container =
        unsafe { aah_status_bar_bridge_debug_uses_scrollable_detail_container() };

    assert_eq!(uses_scrollable_detail_container, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_refreshes_menu_layout_when_the_menu_opens() {
    let refreshes_layout_on_open =
        unsafe { aah_status_bar_bridge_debug_menu_refreshes_layout_on_open() };

    assert_eq!(refreshes_layout_on_open, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_presentation_exposes_compact_utility_panel_tokens() {
    let panel_width = unsafe { aah_status_bar_bridge_debug_panel_width() };
    let switcher_height = unsafe { aah_status_bar_bridge_debug_switcher_height() };
    let hover_radius = unsafe { aah_status_bar_bridge_debug_hover_corner_radius() };
    let action_row_height = unsafe { aah_status_bar_bridge_debug_footer_action_row_height() };
    let icon_column_width = unsafe { aah_status_bar_bridge_debug_footer_icon_column_width() };
    let chevron_column_width = unsafe { aah_status_bar_bridge_debug_footer_chevron_column_width() };
    let footer_horizontal_padding =
        unsafe { aah_status_bar_bridge_debug_footer_action_horizontal_padding() };
    let footer_vertical_padding =
        unsafe { aah_status_bar_bridge_debug_footer_action_vertical_padding() };

    assert_eq!(panel_width, 376);
    assert_eq!(switcher_height, 28);
    assert_eq!(hover_radius, 8);
    assert_eq!(action_row_height, 24);
    assert_eq!(icon_column_width, 14);
    assert_eq!(chevron_column_width, 10);
    assert_eq!(footer_horizontal_padding, 8);
    assert_eq!(footer_vertical_padding, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_presentation_exposes_consistent_padding_tokens() {
    let panel_padding = unsafe { aah_status_bar_bridge_debug_panel_content_padding() };
    let row_horizontal_padding = unsafe { aah_status_bar_bridge_debug_interactive_row_horizontal_padding() };
    let row_vertical_padding = unsafe { aah_status_bar_bridge_debug_interactive_row_vertical_padding() };
    let section_hover_horizontal_padding =
        unsafe { aah_status_bar_bridge_debug_account_section_hover_horizontal_padding() };
    let section_hover_vertical_padding =
        unsafe { aah_status_bar_bridge_debug_account_section_hover_vertical_padding() };

    assert_eq!(panel_padding, 8);
    assert_eq!(row_horizontal_padding, panel_padding);
    assert_eq!(row_vertical_padding, 3);
    assert_eq!(section_hover_horizontal_padding, panel_padding);
    assert_eq!(section_hover_vertical_padding, 6);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_maps_remaining_quota_to_severity_tones() {
    let healthy_tone = unsafe { aah_status_bar_bridge_debug_quota_tone_for_percent(72) };
    let warning_tone = unsafe { aah_status_bar_bridge_debug_quota_tone_for_percent(30) };
    let critical_tone = unsafe { aah_status_bar_bridge_debug_quota_tone_for_percent(10) };

    assert_eq!(healthy_tone, 0);
    assert_eq!(warning_tone, 1);
    assert_eq!(critical_tone, 2);
}
