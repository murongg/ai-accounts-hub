use std::path::PathBuf;
use std::process::Command;

fn run_checked(command: &mut Command) {
    let status = command
        .status()
        .expect("failed to run native build command");
    if !status.success() {
        panic!("native build command failed: {:?}", command);
    }
}

fn compile_macos_bridge() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let objc_object = out_dir.join("status_bar_bridge.o");
    let swift_library = out_dir.join("libaah_status_bar_bridge_swift.a");
    let final_library = out_dir.join("libaah_status_bar_bridge.a");

    println!("cargo:rerun-if-changed=native/macos/AAHStatusBarBridge.h");
    println!("cargo:rerun-if-changed=native/macos/status_bar_bridge.m");
    println!("cargo:rerun-if-changed=native/macos/StatusBarBridgeModels.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarMenuPresentation.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarMenuTheme.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarMenuControls.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarBridgeController.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarBridgeExports.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarStatusItemIcon.swift");
    println!("cargo:rerun-if-changed=native/macos/StatusBarMenuView.swift");

    run_checked(
        Command::new("xcrun")
            .arg("clang")
            .args(["-fobjc-arc", "-c", "native/macos/status_bar_bridge.m"])
            .args(["-I", "native/macos"])
            .args(["-framework", "AppKit"])
            .args(["-framework", "Foundation"])
            .arg("-o")
            .arg(&objc_object),
    );

    let mut swiftc = Command::new("xcrun");
    swiftc
        .arg("swiftc")
        .arg("-parse-as-library")
        .arg("-emit-library")
        .arg("-static")
        .arg("-module-name")
        .arg("AAHStatusBarBridge")
        .arg("-import-objc-header")
        .arg("native/macos/AAHStatusBarBridge.h")
        .arg("-o")
        .arg(&swift_library)
        .arg("native/macos/StatusBarBridgeModels.swift")
        .arg("native/macos/StatusBarMenuPresentation.swift")
        .arg("native/macos/StatusBarMenuTheme.swift")
        .arg("native/macos/StatusBarMenuControls.swift")
        .arg("native/macos/StatusBarBridgeController.swift")
        .arg("native/macos/StatusBarBridgeExports.swift")
        .arg("native/macos/StatusBarStatusItemIcon.swift")
        .arg("native/macos/StatusBarMenuView.swift");
    run_checked(&mut swiftc);

    run_checked(
        Command::new("xcrun")
            .arg("libtool")
            .arg("-static")
            .arg("-o")
            .arg(&final_library)
            .arg(&objc_object)
            .arg(&swift_library),
    );

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=aah_status_bar_bridge");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=QuartzCore");
    println!("cargo:rustc-link-lib=framework=SwiftUI");
}

fn main() {
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=icon-source.svg");
    println!("cargo:rerun-if-changed=icons/icon.icns");
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/icon.png");
    println!("cargo:rerun-if-changed=../public/icon.svg");
    println!("cargo:rerun-if-changed=icons/32x32.png");
    println!("cargo:rerun-if-changed=icons/128x128.png");
    println!("cargo:rerun-if-changed=icons/128x128@2x.png");
    println!("cargo:rerun-if-changed=../src/assets/openai.svg");
    println!("cargo:rerun-if-changed=../src/assets/gemini-color.svg");
    println!("cargo:rerun-if-changed=native/macos/provider-icons/gemini.png");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        compile_macos_bridge();
    }

    tauri_build::build()
}
