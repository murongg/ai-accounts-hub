#[cfg(target_os = "macos")]
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
    fn aah_status_bar_bridge_debug_app_icon_source_variant_for_paths(
        bundle_resource_path: *const i8,
        current_directory_path: *const i8,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_app_icon_is_template() -> c_int;
    fn aah_status_bar_bridge_debug_provider_icon_ready_for_tab(tab_value: c_int) -> c_int;
    fn aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(
        tab_value: c_int,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab_and_paths(
        tab_value: c_int,
        bundle_resource_path: *const i8,
        current_directory_path: *const i8,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_status_item_progress_percent_from_json(
        payload_json: *const i8,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_status_item_uses_dynamic_icon_from_json(
        payload_json: *const i8,
    ) -> c_int;
    fn aah_status_bar_bridge_debug_status_item_accent_clip_width(
        icon_width: f64,
        percent: u8,
    ) -> f64;
    fn aah_status_bar_bridge_debug_status_item_fill_path_variant() -> c_int;
    fn aah_status_bar_bridge_debug_status_item_fill_direction_variant() -> c_int;
    fn aah_status_bar_bridge_debug_status_item_palette_is_monochrome(prefers_dark: c_int) -> c_int;
    fn aah_status_bar_bridge_debug_status_item_opaque_pixel_count(percent: u8) -> c_int;
}

#[cfg(target_os = "macos")]
struct TempIconLayout {
    root: PathBuf,
    resources: PathBuf,
    empty_workdir: PathBuf,
}

#[cfg(target_os = "macos")]
impl TempIconLayout {
    fn new() -> Self {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate should be nested under repo root");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be available")
            .as_nanos();
        let root = env::temp_dir().join(format!("aah-status-bar-icon-layout-{unique}"));
        let resources = root.join("Resources");
        let empty_workdir = root.join("empty-workdir");

        fs::create_dir_all(resources.join("_up_/public")).expect("should create public asset dir");
        fs::create_dir_all(resources.join("_up_/src/assets"))
            .expect("should create vector asset dir");
        fs::create_dir_all(resources.join("native/macos/provider-icons"))
            .expect("should create raster asset dir");
        fs::create_dir_all(&empty_workdir).expect("should create empty workdir");

        copy_fixture(
            &repo_root.join("public/icon.svg"),
            &resources.join("_up_/public/icon.svg"),
        );
        copy_fixture(
            &repo_root.join("src/assets/openai.svg"),
            &resources.join("_up_/src/assets/openai.svg"),
        );
        copy_fixture(
            &repo_root.join("src/assets/claude-color.svg"),
            &resources.join("_up_/src/assets/claude-color.svg"),
        );
        copy_fixture(
            &repo_root.join("src-tauri/native/macos/provider-icons/gemini.png"),
            &resources.join("native/macos/provider-icons/gemini.png"),
        );

        Self {
            root,
            resources,
            empty_workdir,
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for TempIconLayout {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(target_os = "macos")]
fn copy_fixture(from: &Path, to: &Path) {
    fs::copy(from, to).unwrap_or_else(|error| {
        panic!(
            "failed to copy fixture from {} to {}: {error}",
            from.display(),
            to.display()
        )
    });
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
fn native_bridge_builds_provider_icons_for_all_supported_providers() {
    let codex_ready = unsafe { aah_status_bar_bridge_debug_provider_icon_ready_for_tab(1) };
    let claude_ready = unsafe { aah_status_bar_bridge_debug_provider_icon_ready_for_tab(2) };
    let gemini_ready = unsafe { aah_status_bar_bridge_debug_provider_icon_ready_for_tab(3) };

    assert_eq!(codex_ready, 1);
    assert_eq!(claude_ready, 1);
    assert_eq!(gemini_ready, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_uses_vector_codex_and_claude_icons_and_raster_gemini_icon() {
    let codex_variant =
        unsafe { aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(1) };
    let claude_variant =
        unsafe { aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(2) };
    let gemini_variant =
        unsafe { aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab(3) };

    assert_eq!(codex_variant, 1);
    assert_eq!(claude_variant, 1);
    assert_eq!(gemini_variant, 2);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_resolves_release_bundle_icon_paths() {
    let layout = TempIconLayout::new();
    let bundle_resource_path =
        std::ffi::CString::new(layout.resources.to_string_lossy().into_owned())
            .expect("resources path should not contain null bytes");
    let empty_workdir_path =
        std::ffi::CString::new(layout.empty_workdir.to_string_lossy().into_owned())
            .expect("workdir path should not contain null bytes");

    let app_icon_variant = unsafe {
        aah_status_bar_bridge_debug_app_icon_source_variant_for_paths(
            bundle_resource_path.as_ptr(),
            empty_workdir_path.as_ptr(),
        )
    };
    let codex_variant = unsafe {
        aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab_and_paths(
            1,
            bundle_resource_path.as_ptr(),
            empty_workdir_path.as_ptr(),
        )
    };
    let claude_variant = unsafe {
        aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab_and_paths(
            2,
            bundle_resource_path.as_ptr(),
            empty_workdir_path.as_ptr(),
        )
    };
    let gemini_variant = unsafe {
        aah_status_bar_bridge_debug_provider_icon_resource_variant_for_tab_and_paths(
            3,
            bundle_resource_path.as_ptr(),
            empty_workdir_path.as_ptr(),
        )
    };

    assert_eq!(app_icon_variant, 1);
    assert_eq!(codex_variant, 1);
    assert_eq!(claude_variant, 1);
    assert_eq!(gemini_variant, 2);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_decodes_status_item_progress_from_payload_json() {
    let payload = std::ffi::CString::new(
        r#"{"selectedTab":"codex","sections":[],"statusItemProgress":{"providerId":"codex","percent":72,"needsRelogin":false}}"#,
    )
    .expect("payload should not contain null bytes");

    let percent = unsafe {
        aah_status_bar_bridge_debug_status_item_progress_percent_from_json(payload.as_ptr())
    };

    assert_eq!(percent, 72);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_uses_dynamic_status_item_icon_when_progress_is_available() {
    let payload = std::ffi::CString::new(
        r#"{"selectedTab":"codex","sections":[],"statusItemProgress":{"providerId":"codex","percent":72,"needsRelogin":false}}"#,
    )
    .expect("payload should not contain null bytes");

    let uses_dynamic = unsafe {
        aah_status_bar_bridge_debug_status_item_uses_dynamic_icon_from_json(payload.as_ptr())
    };

    assert_eq!(uses_dynamic, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_falls_back_to_static_status_item_icon_when_progress_is_missing() {
    let payload = std::ffi::CString::new(r#"{"selectedTab":"codex","sections":[]}"#)
        .expect("payload should not contain null bytes");

    let uses_dynamic = unsafe {
        aah_status_bar_bridge_debug_status_item_uses_dynamic_icon_from_json(payload.as_ptr())
    };

    assert_eq!(uses_dynamic, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_status_item_accent_clip_width_tracks_percent_bounds() {
    let empty_width = unsafe { aah_status_bar_bridge_debug_status_item_accent_clip_width(18.0, 0) };
    let partial_width =
        unsafe { aah_status_bar_bridge_debug_status_item_accent_clip_width(18.0, 18) };
    let full_width =
        unsafe { aah_status_bar_bridge_debug_status_item_accent_clip_width(18.0, 100) };

    assert_eq!(empty_width, 0.0);
    assert!(partial_width > 0.0);
    assert!(partial_width < full_width);
    assert_eq!(full_width, 18.0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_status_item_progress_uses_full_silhouette_fill_path() {
    let variant = unsafe { aah_status_bar_bridge_debug_status_item_fill_path_variant() };

    assert_eq!(variant, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_status_item_progress_fills_along_the_icon_major_axis() {
    let variant = unsafe { aah_status_bar_bridge_debug_status_item_fill_direction_variant() };

    assert_eq!(variant, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_status_item_palette_is_monochrome_in_light_and_dark_modes() {
    let light_mode = unsafe { aah_status_bar_bridge_debug_status_item_palette_is_monochrome(0) };
    let dark_mode = unsafe { aah_status_bar_bridge_debug_status_item_palette_is_monochrome(1) };

    assert_eq!(light_mode, 1);
    assert_eq!(dark_mode, 1);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_status_item_renders_more_filled_pixels_at_100_than_0_percent() {
    let empty_pixels = unsafe { aah_status_bar_bridge_debug_status_item_opaque_pixel_count(0) };
    let full_pixels = unsafe { aah_status_bar_bridge_debug_status_item_opaque_pixel_count(100) };

    assert!(full_pixels > empty_pixels);
}
