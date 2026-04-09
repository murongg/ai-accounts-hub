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
    fn aah_status_bar_bridge_debug_section_count_from_json(payload_json: *const i8) -> c_int;
    fn aah_status_bar_bridge_debug_selected_tab_value_from_json(payload_json: *const i8) -> c_int;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_decodes_selected_tab_and_section_count() {
    let payload = CString::new(
        r#"{"selectedTab":"codex","sections":[{"id":"codex:1","providerId":"codex","providerTitle":"Codex","email":"a@b.com","subtitle":"Updated 1m ago","plan":"Plus","isActive":true,"needsRelogin":false,"metrics":[],"switchAccountId":null},{"id":"codex:2","providerId":"codex","providerTitle":"Codex","email":"b@b.com","subtitle":"Updated 2m ago","plan":null,"isActive":false,"needsRelogin":false,"metrics":[],"switchAccountId":"acct-2"}]}"#,
    )
    .unwrap();

    let tab = unsafe { aah_status_bar_bridge_debug_selected_tab_value_from_json(payload.as_ptr()) };
    let count = unsafe { aah_status_bar_bridge_debug_section_count_from_json(payload.as_ptr()) };

    assert_eq!(tab, 1);
    assert_eq!(count, 2);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_defaults_invalid_payload_to_empty_values() {
    let payload = CString::new(r#"{"selectedTab":"bogus","sections":"bad"}"#).unwrap();

    let tab = unsafe { aah_status_bar_bridge_debug_selected_tab_value_from_json(payload.as_ptr()) };
    let count = unsafe { aah_status_bar_bridge_debug_section_count_from_json(payload.as_ptr()) };

    assert_eq!(tab, 0);
    assert_eq!(count, 0);
}
