# imago-gpui

Shared visual core for Kosmos **GPUI** apps (Rust). Since KOS-319 the Imago
repo is Rust-only — the `@makekosmos/visuals` Vue/TS library was removed —
and GPUI apps depend on this crate directly.

Palette, theme helpers and chrome were seeded from agenda-gpui
(`src/palettes.rs`, `src/theme.rs`, `src/chrome.rs`), so GPUI shells render
the Agenda look by default. There is intentionally **no CSS codegen
pipeline** — `src/palettes.rs` is a normal source file, edited by hand.

## Contents

| Module | What |
|---|---|
| `palettes` | `THEMES` table + `ThemeDef` (zeron-derived light/dark pairs) |
| `theme` | `Palette`, `pal()`/`set_theme()`/`set_mode()`, token fns (`BG()`…), color helpers (`c`/`rgba`/`mix`/`lerp`), easings, `apply(cx)` to install the palette into `gpui-component` |
| `chrome` | `sidebar()`/`sidebar_body()`/`sidebar_footer()`/`titlebar()`/`SidebarItem` — Agenda sidebar spec (h32 rows, fg-alpha glyphs, 12%/8% fills) |
| `button` | `primary`/`secondary`/`ghost`/`danger`/`success` over `gpui-component::Button` |

## Depending on it

The repo root has a Cargo workspace (`members = ["crates/*"]`), so
consumers use a git dependency pinned by rev — the same rev as
`mundus-gpui-kit` when both are used:

```toml
imago-gpui = { git = "https://github.com/makekosmos/imago.git", rev = "<sha>" }
```

App wiring:

```rust,ignore
gpui::application().with_assets(imago_gpui::assets::Assets).run(|cx: &mut App| {
    gpui_component::init(cx);
    cx.text_system().add_fonts(imago_gpui::assets::font_bytes()).unwrap();
    imago_gpui::theme::apply(cx);
    // ...
});
```

The bundled Hugeicons SVGs come from `@hugeicons/core-free-icons` 4.3.2.
Inter and the settings/sidebar/help glyphs match Agenda GPUI at
`27b737a131494a51a68bd012ecf32a58854afa4a`. Applications choose icon paths
through `gpui_component::Icon::default().path("icons/database.svg")`.
The theme uses a 16px rem for Agenda's spacing; body and sidebar text use
explicit 13px sizes. Applications should set `.font_family("Inter")` on
their root. Asset and font registration are required; applying colors alone
does not install them.

## Local gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

(From the repo root.)
