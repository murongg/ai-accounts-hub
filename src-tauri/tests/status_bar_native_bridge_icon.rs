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
    fn aah_status_bar_bridge_debug_icon_ready() -> c_int;
    fn aah_status_bar_bridge_debug_app_icon_source_variant() -> c_int;
    fn aah_status_bar_bridge_debug_app_icon_is_template() -> c_int;
    fn aah_status_bar_bridge_debug_provider_icon_ready_for_tab(tab_value: c_int) -> c_int;
    fn aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(tab_value: c_int) -> c_int;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_builds_a_status_item_icon() {
    let ready = unsafe { aah_status_bar_bridge_debug_icon_ready() };
    assert_eq!(ready, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_prefers_explicit_app_icon_assets_over_runtime_fallbacks() {
    let source_variant = unsafe { aah_status_bar_bridge_debug_app_icon_source_variant() };
    assert_eq!(source_variant, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_uses_a_template_menubar_icon() {
    let is_template = unsafe { aah_status_bar_bridge_debug_app_icon_is_template() };
    assert_eq!(is_template, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_builds_provider_icons_for_codex_and_gemini() {
    let codex_ready = unsafe { aah_status_bar_bridge_debug_provider_icon_ready_for_tab(1) };
    let gemini_ready = unsafe { aah_status_bar_bridge_debug_provider_icon_ready_for_tab(2) };

    assert_eq!(codex_ready, 1);
    assert_eq!(gemini_ready, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_uses_vector_codex_icon_and_raster_gemini_icon() {
    let codex_variant = unsafe { aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(1) };
    let gemini_variant = unsafe { aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(2) };

    assert_eq!(codex_variant, 1);
    assert_eq!(gemini_variant, 2);
}
