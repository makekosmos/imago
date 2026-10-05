//! Appearance previews extracted from Cortex (Zeron MIT visuals, 2026 Wing).
//! Only rendering and interaction live here; callers own selection/persistence.
use crate::{
    settings::UiStyle,
    theme::{c, rgba, Palette},
};
use ::gpui::{prelude::*, *};
use std::rc::Rc;

fn bar(fraction: f32, color: Hsla) -> Div {
    div()
        .h(px(5.))
        .w(relative(fraction))
        .rounded(px(3.))
        .bg(color)
}

pub fn miniature(palette: &Palette) -> Div {
    let line = rgba(palette.fg, 0.22);
    let strong = rgba(palette.fg, 0.34);
    div()
        .size_full()
        .flex()
        .rounded(px(6.))
        .bg(c(palette.card))
        .child(
            div()
                .w(px(44.))
                .h_full()
                .flex_none()
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(px(7.))
                .px(px(8.))
                .pt(px(14.))
                .child(bar(0.70, strong))
                .child(bar(1., line))
                .child(bar(0.85, line))
                .child(bar(1., line)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .my(px(8.))
                .mr(px(8.))
                .rounded(px(6.))
                .border_1()
                .border_color(c(palette.border))
                .bg(c(palette.bg))
                .overflow_hidden()
                .flex()
                .flex_col()
                .gap(px(7.))
                .p(px(10.))
                .child(bar(0.62, strong))
                .child(bar(0.88, line))
                .child(bar(0.76, line))
                .child(bar(0.52, line)),
        )
}

pub fn system_preview(light: &Palette, dark: &Palette) -> Div {
    div()
        .size_full()
        .flex()
        .rounded(px(6.))
        .overflow_hidden()
        .children([light, dark].into_iter().map(|palette| {
            div()
                .w_1_2()
                .h_full()
                .bg(c(palette.bg))
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(7.))
                .child(bar(0.62, rgba(palette.fg, 0.34)))
                .child(bar(0.88, rgba(palette.fg, 0.22)))
                .child(bar(0.7, rgba(palette.fg, 0.22)))
        }))
}

type SelectHandler = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct ModeTile {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    preview: Div,
    focus: FocusHandle,
    selected: bool,
    disabled: bool,
    style: UiStyle,
    on_select: Option<SelectHandler>,
}

impl ModeTile {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        preview: Div,
        focus: FocusHandle,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            preview,
            focus,
            selected: false,
            disabled: false,
            style: UiStyle::default(),
            on_select: None,
        }
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }
    pub fn style(mut self, style: UiStyle) -> Self {
        self.style = style;
        self
    }
    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}
impl gpui_component::Disableable for ModeTile {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for ModeTile {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let selector = self.id.to_string();
        let style = self.style;
        let enabled = !self.disabled;
        let focus = self.focus;
        let click = self.on_select.clone();
        let keyboard = self.on_select;
        let edge = if self.selected {
            style.accent
        } else {
            style.border
        };
        let caption = if self.selected {
            style.accent
        } else {
            style.muted
        };
        div()
            .id(self.id)
            .debug_selector(move || selector.clone())
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(8.))
            .role(Role::Button)
            .aria_label(format!("Режим темы: {}", self.label))
            .aria_selected(self.selected)
            .track_focus(&focus)
            .tab_index(if enabled { 0 } else { -1 })
            .when(enabled, |tile| tile.cursor_pointer())
            .when(!enabled, |tile| tile.opacity(0.5))
            .focus_visible(move |s| s.bg(rgba(style.foreground, 0.05)).rounded_md())
            .on_click(move |_, window, cx| {
                if enabled {
                    window.focus(&focus, cx);
                    if let Some(handler) = &click {
                        handler(window, cx);
                    }
                }
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if enabled && matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    if let Some(handler) = &keyboard {
                        handler(window, cx);
                    }
                    cx.stop_propagation();
                }
            })
            .child(
                div()
                    .h(px(148.))
                    .flex_none()
                    .w_full()
                    .rounded(px(6.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(c(edge))
                    .child(self.preview),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(style.text_px(13.))
                    .line_height(style.text_px(16.))
                    .font_weight(if self.selected {
                        FontWeight::MEDIUM
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(c(caption))
                    .when_some(self.icon, |d, path| {
                        d.child(
                            gpui_component::Icon::default()
                                .path(path)
                                .size(px(16.))
                                .text_color(c(caption)),
                        )
                    })
                    .child(self.label),
            )
    }
}
