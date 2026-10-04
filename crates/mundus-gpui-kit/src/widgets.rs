//! Shared chrome: frameless window controls and the field/view primitives
//! (re-exported here). App-shell pieces (sidebar, titlebar, banner) stay in
//! the consuming app — they bind to its root entity and nav types; shell
//! primitives come from `imago_gpui::chrome`.
pub use crate::fields::*;

// --- Chrome -----------------------------------------------------------------

/// Frameless caption buttons (minimize / close). Delegates to the imago
/// chrome primitive so the emitted role, Russian accessible names and
/// hover colors stay identical across GPUI apps.
pub fn render_window_controls() -> impl ::gpui::IntoElement {
    imago_gpui::chrome::window_controls()
}
