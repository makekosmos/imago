//! Shared button variants on top of `gpui-component::Button`.
//!
//! | Variant       | helper       | gpui-component mapping |
//! |---------------|--------------|------------------------|
//! | primary       | `primary`    | `.primary()`           |
//! | surface       | `secondary`  | `.secondary()`         |
//! | ghost         | `ghost`      | `.outline()` (border + faint fill) |
//! | danger        | `danger`     | `.danger().outline()` (tinted outline) |
//! | success       | `success`    | `.success().outline()` |
//!
//! The variant *colors* are the shared palette installed by
//! `theme::apply` — these helpers only own the variant choice, so buttons
//! look identical across GPUI apps without each app re-deriving it.
//!
//! Accessibility: `gpui-component::Button` always emits
//! `Role::Button`/`Role::Link` and names itself from `.label(...)`, so a
//! labeled button needs no extra wiring. Icon-only buttons (no visible
//! text) must carry `.accessibility_label(<Russian name>)` — they have no
//! label to derive from.

use gpui::ElementId;
use gpui_component::button::{Button, ButtonVariants};

/// Imago button variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonKind {
    /// `.button-primary` — accent fill (call-to-action).
    Primary,
    /// `.button-surface` — neutral high-emphasis fill.
    #[default]
    Secondary,
    /// `.button-ghost` — bordered faint fill.
    Ghost,
    /// `.button-danger` — destructive outline.
    Danger,
    /// `.button-success` — success outline.
    Success,
}

/// `Button::new(id)` with the Imago variant applied.
pub fn button(id: impl Into<ElementId>, kind: ButtonKind) -> Button {
    match kind {
        ButtonKind::Primary => primary(id),
        ButtonKind::Secondary => secondary(id),
        ButtonKind::Ghost => ghost(id),
        ButtonKind::Danger => danger(id),
        ButtonKind::Success => success(id),
    }
}

/// `.button-primary`: accent background, accent-foreground text.
pub fn primary(id: impl Into<ElementId>) -> Button {
    Button::new(id).primary()
}

/// `.button-surface`: filled neutral control (gpui "secondary").
pub fn secondary(id: impl Into<ElementId>) -> Button {
    Button::new(id).secondary()
}

/// `.button-ghost`: border + faint foreground fill (gpui "outline" — a
/// border with a translucent fill, unlike gpui's borderless `.ghost()`).
pub fn ghost(id: impl Into<ElementId>) -> Button {
    Button::new(id).outline()
}

/// `.button-danger`: destructive text/border, transparent at rest.
pub fn danger(id: impl Into<ElementId>) -> Button {
    Button::new(id).danger().outline()
}

/// `.button-success`: success-tinted outline.
pub fn success(id: impl Into<ElementId>) -> Button {
    Button::new(id).success().outline()
}
