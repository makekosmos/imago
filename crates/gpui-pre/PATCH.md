# Kosmos patch: gpui-pre 0.3.9 manifest no-op

Forked from crates.io `gpui-pre` 0.3.9. gpui-kit 0.8 keeps gpui-pre as a compatibility dependency, and its default features enable `windows-manifest`. Kosmos GUI hosts embed their own RT_MANIFEST resource; gpui-pre's second resource triggers MSVC CVT1100/LNK1123.

Changes:
- remove `windows-manifest` from default features;
- leave the feature name valid for compatibility, but make `build.rs` omit gpui-pre's manifest unconditionally.

This is independent of gpui-fast's `windows-manifest-provided-by-host`: that feature suppresses gpui-fast's resource only. The legacy gpui-pre compatibility crate needs its own no-op because gpui-kit 0.8 still enables it.

## macOS GPUIView embedding spike

`../gpui-pre-macos` is a source copy of the exact `gpui-pre-macos 0.3.9`
selected by this workspace's lockfile. The root `[patch.crates-io]` selects
that fork for GPUI dependencies.

Changes are limited to `src/events.rs` and `src/window.rs`:
- Translate mouse, scroll, drag, pressure, and IME coordinates against the
  embedded `GPUIView` bounds and its origin in the window.
- Report GPUIView content size and IME screen frames from that view.
- Accept first responder so clicks in a native split can return keyboard
  input to GPUI.
- Keep a GPUI view embedded in a native layout transparent even when the GPUI
  root requests an opaque background; ordinary GPUI windows keep their existing
  appearance behavior.

The spike-only `imago-gpui-native-split` crate owns the AppKit split, toolbar,
and source list. It does not create `NSApplication` or a run loop.
