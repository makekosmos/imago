# Kosmos patch: gpui-fast 0.1.10

Forked from crates.io `gpui-fast` 0.1.10. The only change in this patch is in `src/platform/test/window.rs`: TestWindow's `PlatformWindow::a11y_init` immediately invokes `A11yCallbacks::activation`, because headless test windows have no platform accessibility adapter. This enables `Window::debug_a11y_tree_json` in UI tests, matching the Kosmos hook previously carried in the gpui-pre fork.

No production platform behavior is changed by this patch. Package source metadata and upstream license are retained.

This fork is temporary: remove the local `[patch.crates-io]` override and delete `crates/gpui-fast` after the upstream gpui-fast PR accepting `TestWindow::a11y_init` is released (or otherwise available to this dependency pin).
