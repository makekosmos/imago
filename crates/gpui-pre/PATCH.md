# Kosmos patch: gpui-pre 0.3.9 manifest no-op

Forked from crates.io `gpui-pre` 0.3.9. gpui-kit 0.8 keeps gpui-pre as a compatibility dependency, and its default features enable `windows-manifest`. Kosmos GUI hosts embed their own RT_MANIFEST resource; gpui-pre's second resource triggers MSVC CVT1100/LNK1123.

Changes:
- remove `windows-manifest` from default features;
- leave the feature name valid for compatibility, but make `build.rs` omit gpui-pre's manifest unconditionally.

This is independent of gpui-fast's `windows-manifest-provided-by-host`: that feature suppresses gpui-fast's resource only. The legacy gpui-pre compatibility crate needs its own no-op because gpui-kit 0.8 still enables it.
