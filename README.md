# imago

The Kosmos GPUI design layer, as a Rust workspace:

- `crates/imago-gpui` — palettes, theme tokens and runtime theme helpers plus
  shared button/sidebar chrome, settings layouts, appearance previews,
  switches and About/diagnostics blocks extracted from Cortex.
  See [shared settings UI](docs/shared-settings.md) for API and local integration.
- `crates/mundus-gpui-kit` — view primitives and JSON accessors (`fields`),
  shell widgets (`widgets`), theme glue (`theme`) and the Engine lock/RPC
  client (`engine`, `engine_ws`, `engine_error`), merged in from the archived
  `makekosmos/kosmos-gpui-kit` repository (KOS-319).

Consumers (`cortex/manager-gpui`, `agenda-gpui`, `memoria-gpui`) depend on the
crates by pinned git rev; both crates must come from the same rev so the
lockfile holds a single imago source.

## Verification

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

The same commands run as lefthook pre-push hooks and as the Quality workflow
on every PR and push to main, on Linux, Windows and macOS — the platforms
the consumers ship on. The pinned toolchain lives in `toolchain.json` and
`rust-toolchain.toml` (rust 1.95.0, same pin as cortex); CI fails if they
disagree.
