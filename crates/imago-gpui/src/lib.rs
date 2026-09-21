//! # imago-gpui
//!
//! Shared visual core for Kosmos GPUI apps. The palette and chrome were
//! seeded from agenda-gpui (`src/palettes.rs` / `src/theme.rs` /
//! `src/chrome.rs`), so every GPUI shell renders the same colors and
//! sidebar/button chrome — Agenda's look is the reference.
//!
//! This crate is NOT part of the `@makekosmos/visuals` npm package: the
//! tarball keeps shipping only Vue/TS artifacts, GPUI apps depend on this
//! crate directly (git rev dependency — see `README.md`).
//!
//! * [`palettes`] — theme table (`THEMES`, [`ThemeDef`]); seeded values,
//!   maintained by hand (no codegen pipeline).
//! * [`theme`] — active palette access (`pal()`, `BG()`/`FG()`/…), color
//!   helpers (`c`/`rgba`/`mix`/`lerp`), easings, and [`theme::apply`] to
//!   install the palette into `gpui-component`.
//! * [`chrome`] — sidebar shell/items and content titlebar (Agenda spec).
//! * [`button`] — button variant helpers over `gpui-component::Button`.
//!
//! ```ignore
//! gpui::application().run(|cx: &mut App| {
//!     gpui_component::init(cx);
//!     imago_gpui::theme::apply(cx); // palette → gpui-component theme
//!     ...
//! });
//! ```

pub mod button;
pub mod chrome;
pub mod palettes;
pub mod theme;

pub use palettes::{ThemeDef, THEMES};
pub use theme::Palette;
