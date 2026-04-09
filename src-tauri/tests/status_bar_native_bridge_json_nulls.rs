#[cfg(target_os = "macos")]
use std::ffi::CString;

#[cfg(target_os = "macos")]
use std::os::raw::{c_char, c_int};

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
#[link(name = "aah_status_bar_bridge", kind = "static")]
unsafe extern "C" {
    fn aah_status_bar_bridge_optional_string_length_from_json(
        payload_json: *const c_char,
        field_name: *const c_char,
    ) -> c_int;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_treats_null_optional_strings_as_empty() {
    let payload = CString::new(r#"{"sections":[{"plan":null,"switchAccountId":null}]}"#).unwrap();
    let plan = CString::new("plan").unwrap();
    let switch_account_id = CString::new("switchAccountId").unwrap();

    let plan_length =
        unsafe { aah_status_bar_bridge_optional_string_length_from_json(payload.as_ptr(), plan.as_ptr()) };
    let switch_length = unsafe {
        aah_status_bar_bridge_optional_string_length_from_json(payload.as_ptr(), switch_account_id.as_ptr())
    };

    assert_eq!(plan_length, 0);
    assert_eq!(switch_length, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_preserves_non_null_optional_strings() {
    let payload =
        CString::new(r#"{"sections":[{"plan":"Plus","switchAccountId":"account-123"}]}"#).unwrap();
    let plan = CString::new("plan").unwrap();
    let switch_account_id = CString::new("switchAccountId").unwrap();

    let plan_length =
        unsafe { aah_status_bar_bridge_optional_string_length_from_json(payload.as_ptr(), plan.as_ptr()) };
    let switch_length = unsafe {
        aah_status_bar_bridge_optional_string_length_from_json(payload.as_ptr(), switch_account_id.as_ptr())
    };

    assert_eq!(plan_length, 4);
    assert_eq!(switch_length, 11);
}
