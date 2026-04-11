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
#[link(name = "aah_status_bar_bridge", kind = "static")]
unsafe extern "C" {
    fn aah_status_bar_bridge_debug_text_palette_adapts_to_appearance() -> c_int;
    fn aah_status_bar_bridge_debug_hosting_view_matches_requested_system_appearance(
        prefers_dark: c_int,
    ) -> c_int;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_uses_appearance_aware_text_colors() {
    let adapts = unsafe { aah_status_bar_bridge_debug_text_palette_adapts_to_appearance() };
    assert_eq!(adapts, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_hosting_view_can_follow_light_system_appearance() {
    let matches =
        unsafe { aah_status_bar_bridge_debug_hosting_view_matches_requested_system_appearance(0) };
    assert_eq!(matches, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_hosting_view_can_follow_dark_system_appearance() {
    let matches =
        unsafe { aah_status_bar_bridge_debug_hosting_view_matches_requested_system_appearance(1) };
    assert_eq!(matches, 1);
}
