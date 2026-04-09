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
    fn aah_status_bar_bridge_needs_main_queue_hop(is_main_thread: c_int) -> c_int;
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_skips_main_queue_hop_on_main_thread() {
    let needs_hop = unsafe { aah_status_bar_bridge_needs_main_queue_hop(1) };
    assert_eq!(needs_hop, 0);
}

#[cfg(target_os = "macos")]
#[test]
fn native_bridge_hops_to_main_queue_off_main_thread() {
    let needs_hop = unsafe { aah_status_bar_bridge_needs_main_queue_hop(0) };
    assert_eq!(needs_hop, 1);
}
