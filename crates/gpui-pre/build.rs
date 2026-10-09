#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

fn main() {
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

    if target_os == "windows" {
        #[cfg(feature = "windows-manifest")]
        embed_resource();
    }
}

// Local patch (KOS-328): the `windows-manifest` feature is force-enabled by
// gpui-pre-platform, so it cannot be turned off from the outside. The apps
// embed their own windows/app.manifest (Common Controls v6 + PerMonitorV2,
// superset of gpui.manifest.xml) via their own build.rs; embedding a second
// RT_MANIFEST id=1 fails to link (CVTRES CVT1100 / LNK1123). Keep the
// feature valid but make the embed a no-op.
#[cfg(feature = "windows-manifest")]
fn embed_resource() {}
