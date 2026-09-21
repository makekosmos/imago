//! Shared chrome primitives for Kosmos GPUI apps — the sidebar shell and
//! items plus the content titlebar, seeded from agenda-gpui's chrome
//! (`.kosmos-sidebar-btn`: h32, pl10/pr8, r8, fs13/lh15, fg-alpha glyphs,
//! fg 12% active / 8% hover fills).
//!
//! App-specific bits (which items, where clicks route) stay in the app:
//! items take an `on_click` handler so the crate owns only the look.

use std::rc::Rc;

use gpui::{
    div, px, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window,
};
use gpui_component::scroll::{Scrollable, ScrollableElement};
use gpui_component::{Icon, Sizable};

use crate::theme::{c, rgba, BORDER, FG, SIDEBAR_BG, SIDEBAR_DIVIDER};

/// Sidebar width shared by Agenda/Manager (`SIDEBAR_W` in both shells).
pub const SIDEBAR_W: f32 = 240.0;
/// Titlebar strip height (sidebar header + content header).
pub const TITLEBAR_H: f32 = 40.0;
/// Sidebar item row height.
pub const ITEM_H: f32 = 32.0;

/// Sidebar shell: full-height column, `sidebar_bg` fill, right hairline in
/// `sidebar_divider`.
pub fn sidebar() -> Div {
    div()
        .w(px(SIDEBAR_W))
        .h_full()
        .flex_shrink_0()
        .bg(c(SIDEBAR_BG()))
        .border_r_1()
        .border_color(c(SIDEBAR_DIVIDER()))
        .flex()
        .flex_col()
        .overflow_hidden()
}

/// Sidebar header row (`TITLEBAR_H` tall). Usually a window drag region —
/// apply `.window_control_area(WindowControlArea::Drag)` at the call site.
pub fn sidebar_titlebar() -> Div {
    div()
        .h(px(TITLEBAR_H))
        .flex_shrink_0()
        .flex()
        .items_center()
        .px_3()
}

/// Scrollable sidebar body.
pub fn sidebar_body() -> Scrollable<Div> {
    div()
        .flex_1()
        .overflow_y_scrollbar()
        .py_1()
        .flex()
        .flex_col()
        .gap(px(1.))
}

/// Sidebar footer strip (version text, icon buttons) with a top hairline.
pub fn sidebar_footer() -> Div {
    div()
        .flex_shrink_0()
        .flex()
        .items_center()
        .p_3()
        .border_t_1()
        .border_color(c(SIDEBAR_DIVIDER()))
}

/// Content-area titlebar: `TITLEBAR_H` tall, bottom hairline in `border`.
/// Apply `WindowControlArea::Drag` at the call site.
pub fn titlebar() -> Div {
    div()
        .h(px(TITLEBAR_H))
        .flex_shrink_0()
        .flex()
        .items_center()
        .px_4()
        .border_b_1()
        .border_color(c(BORDER()))
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One `.kosmos-sidebar-btn` row: h32, pl10 pr8, r8, gap6, fs13 lh15.
///
/// * rest: icon + label at `fg` 60%
/// * hover: `fg` 8% fill, label brightens to full `fg`
/// * active: `fg` 12% fill, full-alpha glyphs
///
/// Attach behavior with `.on_click(...)`; the handler signature is the
/// plain gpui one (`Fn(&ClickEvent, &mut Window, &mut App)`), so apps wire
/// their own entity updates via `cx.weak_entity()`.
pub struct SidebarItem {
    id: ElementId,
    icon: Option<Icon>,
    label: Option<SharedString>,
    active: bool,
    on_click: Option<ClickHandler>,
}

impl SidebarItem {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: None,
            active: false,
            on_click: None,
        }
    }

    /// Row label (fs13, single-line ellipsis).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Leading icon rendered at 16px in the row's glyph color. Accepts
    /// `gpui::IconName`, `gpui_component::Icon`, etc.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl IntoElement for SidebarItem {
    // Concrete `Stateful<Div>` so callers can keep chaining (`.on_click`,
    // `.tooltip`, …) on the rendered element.
    type Element = Stateful<Div>;

    fn into_element(self) -> Stateful<Div> {
        let fg = FG();
        let glyph = rgba(fg, if self.active { 1.0 } else { 0.6 });

        let mut el = div()
            .id(self.id)
            .h(px(ITEM_H))
            .w_full()
            .flex()
            .flex_none()
            .items_center()
            .gap_1p5()
            .pl(px(10.))
            .pr_2()
            .rounded_lg()
            .text_size(px(13.))
            .line_height(px(15.))
            .cursor_pointer()
            .text_color(glyph);
        if self.active {
            el = el.bg(rgba(fg, 0.12));
        } else {
            el = el.hover(|s| s.bg(rgba(fg, 0.08)).text_color(rgba(fg, 1.0)));
        }

        if let Some(icon) = self.icon {
            el = el.child(icon.with_size(px(16.)).text_color(glyph));
        }
        if let Some(label) = self.label {
            el = el.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(label),
            );
        }
        if let Some(on_click) = self.on_click {
            el = el.on_click(move |ev, window, cx| on_click(ev, window, cx));
        }
        el
    }
}

/// Convenience wrapper when the item is already fully configured.
pub fn sidebar_item(
    id: impl Into<ElementId>,
    icon: impl Into<Icon>,
    label: impl Into<SharedString>,
    active: bool,
) -> SidebarItem {
    SidebarItem::new(id).icon(icon).label(label).active(active)
}
