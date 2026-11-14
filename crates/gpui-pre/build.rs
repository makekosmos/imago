#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(gles)");

    // Leak detection does bookkeeping on every entity handle, so `test-support`
    // does not enable it: that feature must stay cheap enough to compile into
    // every build. CI sets `GPUI_LEAK_DETECTION`; the `leak-detection` feature
    // turns it on explicitly.
    println!("cargo::rustc-check-cfg=cfg(gpui_leak_detection)");
    println!("cargo::rerun-if-env-changed=GPUI_LEAK_DETECTION");
    let requested_by_environment =
        std::env::var_os("GPUI_LEAK_DETECTION").is_some_and(|value| !value.is_empty());
    if requested_by_environment || std::env::var_os("CARGO_FEATURE_LEAK_DETECTION").is_some() {
        println!("cargo::rustc-cfg=gpui_leak_detection");
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Hot reloading is limited to debug builds so the feature can't ship in release binaries,
    // and to the platforms whose patch transport is implemented.
    println!("cargo::rustc-check-cfg=cfg(gpui_hot_reload)");
    if std::env::var_os("CARGO_FEATURE_HOT_RELOAD").is_some()
        && std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some()
        && matches!(target_os.as_str(), "macos" | "linux" | "windows" | "ios")
    {
        println!("cargo::rustc-cfg=gpui_hot_reload");
    }

    // gpui-kit keeps gpui-pre as a compatibility dependency. The host embeds its
    // own app manifest; embedding another RT_MANIFEST id=1 causes
    // CVT1100/LNK1123. Keep the feature for upstream compatibility, but do not
    // embed gpui-pre's bundled manifest. Kosmos patch.
}
