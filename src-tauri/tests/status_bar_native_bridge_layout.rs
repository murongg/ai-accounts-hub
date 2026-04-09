#[cfg(target_os = "macos")]
use std::os::raw::c_double;

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
    fn aah_status_bar_bridge_panel_height_for_content_height(content_height: c_double) -> c_double;
    fn aah_status_bar_bridge_panel_height_clamped_to_available_height(
        content_height: c_double,
        available_height: c_double,
    ) -> c_double;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_panel_height_shrinks_to_match_content() {
    let height = unsafe { aah_status_bar_bridge_panel_height_for_content_height(0.0) };
    assert_eq!(height, 0.0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_panel_height_matches_content_height() {
    let height = unsafe { aah_status_bar_bridge_panel_height_for_content_height(300.0) };
    assert_eq!(height, 300.0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_panel_height_uses_natural_height_when_screen_has_space() {
    let height = unsafe { aah_status_bar_bridge_panel_height_clamped_to_available_height(300.0, 600.0) };
    assert_eq!(height, 300.0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_panel_height_clamps_to_available_height_when_needed() {
    let height = unsafe { aah_status_bar_bridge_panel_height_clamped_to_available_height(600.0, 420.0) };
    assert_eq!(height, 420.0);
}
