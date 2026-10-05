//! Settings selector, ported from Cortex's `appearance::selector` (itself
//! ported from Zeron's MIT-licensed settings widgets). The host snapshot
//! still owns every value; this only renders and resolves local interaction.
use super::{c, rgba, AppearanceEditor, Menu};
use ::gpui::{prelude::*, *};
use gpui_component::scroll::ScrollableElement;
use serde_json::json;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct OptionItem {
    pub label: String,
    pub value: String,
    pub palette: Option<(u32, u32, u32, u32)>,
}

impl OptionItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            palette: None,
        }
    }

    pub fn palette(mut self, surface: u32, background: u32, accent: u32, border: u32) -> Self {
        self.palette = Some((surface, background, accent, border));
        self
    }
}

fn chip((surface, background, accent, border): (u32, u32, u32, u32)) -> Div {
    div()
        .w(px(30.))
        .h(px(18.))
        .flex_none()
        .rounded(px(5.))
        .overflow_hidden()
        .border_1()
        .border_color(c(border))
        .flex()
        .child(div().w_1_3().h_full().bg(c(surface)))
        .child(div().w_1_3().h_full().bg(c(background)))
        .child(div().w_1_3().h_full().bg(c(accent)))
}

fn commit(
    editor: &mut AppearanceEditor,
    kind: Menu,
    option: &OptionItem,
    cx: &mut Context<AppearanceEditor>,
) {
    editor.open_menu = None;
    editor.font_menu_open = false;
    let params = match kind {
        Menu::LightTheme => json!({"light_theme": option.value}),
        Menu::DarkTheme => json!({"dark_theme": option.value}),
        Menu::FontFamily => json!({"font_family": option.value}),
        Menu::FontSize => {
            let Ok(size) = option.value.parse::<f32>() else {
                return;
            };
            json!({"font_size": size})
        }
        Menu::Material => json!({"material": option.value}),
    };
    editor.patch(params, cx);
}

fn close(editor: &mut AppearanceEditor, cx: &mut Context<AppearanceEditor>) {
    let dismissed = editor.open_menu;
    editor.open_menu = None;
    editor.font_menu_open = false;
    editor.menu_dismissed_at = dismissed.map(|kind| (kind, Instant::now()));
    cx.notify();
}

fn open(
    editor: &mut AppearanceEditor,
    kind: Menu,
    selected: usize,
    window: &mut Window,
    cx: &mut Context<AppearanceEditor>,
) {
    if !editor.editable() {
        return;
    }
    editor.open_menu = Some(kind);
    editor.font_menu_open = kind == Menu::FontFamily;
    editor.menu_highlighted = selected;
    editor.menu_query.clear();
    editor.menu_generation += 1;
    editor.menu_dismissed_at = None;
    if kind == Menu::FontFamily {
        let input = editor.font_search_input(window, cx);
        input.update(cx, |state, cx| state.set_value("", window, cx));
        window.focus(&input.read(cx).focus_handle(cx), cx);
    }
    cx.notify();
}

fn key_down(
    editor: &mut AppearanceEditor,
    kind: Menu,
    key: &str,
    selected: usize,
    options: &[OptionItem],
    window: &mut Window,
    cx: &mut Context<AppearanceEditor>,
) -> bool {
    let is_open = editor.editable()
        && (editor.open_menu == Some(kind) || (kind == Menu::FontFamily && editor.font_menu_open));
    if !is_open {
        if matches!(key, "up" | "down" | "enter" | "space") {
            open(editor, kind, selected, window, cx);
            return true;
        }
        return false;
    }
    let count = options.len();
    match key {
        "up" => editor.menu_highlighted = editor.menu_highlighted.saturating_sub(1),
        "down" => {
            editor.menu_highlighted = (editor.menu_highlighted + 1).min(count.saturating_sub(1))
        }
        "home" => editor.menu_highlighted = 0,
        "end" => editor.menu_highlighted = count.saturating_sub(1),
        "escape" => {
            close(editor, cx);
            return true;
        }
        "enter" | "space" if kind != Menu::FontFamily || key == "enter" => {
            if let Some(option) = options.get(editor.menu_highlighted) {
                commit(editor, kind, option, cx);
            } else {
                close(editor, cx);
            }
            return true;
        }
        _ => return false,
    }
    cx.notify();
    true
}

pub struct SelectSpec {
    pub kind: Menu,
    pub id: &'static str,
    pub aria_label: &'static str,
    pub current: String,
    pub options: Vec<OptionItem>,
    pub trigger_width: f32,
    pub menu_width: f32,
    pub heading: Option<&'static str>,
}

/// Source geometry: trigger h32/r8/pl10/pr8/gap8, 12.5px fixed type;
/// theme 218/260, font 220, size 128. The popover stays out of page flow.
pub fn select(
    editor: &mut AppearanceEditor,
    spec: SelectSpec,
    window: &mut Window,
    cx: &mut Context<AppearanceEditor>,
) -> Stateful<Div> {
    let SelectSpec {
        kind,
        id,
        aria_label,
        current,
        options,
        trigger_width,
        menu_width,
        heading,
    } = spec;
    let style = editor.style;
    let enabled = editor.editable();
    let is_open = enabled
        && (editor.open_menu == Some(kind) || (kind == Menu::FontFamily && editor.font_menu_open));
    let selected = options.iter().position(|item| item.value == current);
    let selected_ix = selected.unwrap_or(0);
    let current_value = current.clone();
    let label = if editor.ready {
        selected
            .and_then(|ix| options.get(ix))
            .map(|item| item.label.clone())
            .unwrap_or(current)
    } else {
        "Загрузка…".to_owned()
    };
    let query = if kind == Menu::FontFamily && is_open {
        let input = editor.font_search_input(window, cx);
        input.read(cx).value().to_lowercase()
    } else {
        String::new()
    };
    let mut visible: Vec<OptionItem> = options
        .into_iter()
        .filter(|option| option.label.to_lowercase().contains(&query))
        .collect();
    if kind == Menu::FontFamily && !query.is_empty() {
        visible.sort_by_key(|option| !option.label.to_lowercase().starts_with(&query));
    }
    if editor.menu_query != query {
        editor.menu_query = query;
        editor.menu_highlighted = 0;
    }
    editor.menu_highlighted = editor.menu_highlighted.min(visible.len().saturating_sub(1));
    let highlighted = editor.menu_highlighted.min(visible.len().saturating_sub(1));
    let focus = editor
        .tile_focus
        .entry(id.to_owned())
        .or_insert_with(|| cx.focus_handle())
        .clone();
    let open_color = rgba(style.foreground, 0.10);
    let base_color = rgba(style.foreground, 0.06);
    let options_for_click = visible.clone();
    let mut trigger = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .relative()
        .flex_none()
        .max_w_full()
        .w(px(trigger_width))
        .h(px(32.))
        .pl(px(10.))
        .pr(px(8.))
        .rounded(px(8.))
        .border_1()
        .border_color(transparent_black())
        .bg(if is_open { open_color } else { base_color })
        .hover(|el| el.bg(open_color))
        .flex()
        .items_center()
        .gap(px(8.))
        .text_size(style.text_px(12.5))
        .text_color(c(style.foreground))
        .role(Role::Button)
        .aria_label(format!("{aria_label}: {label}"))
        .aria_expanded(is_open)
        .track_focus(&focus)
        .tab_index(0)
        .when(enabled, |el| el.cursor_pointer())
        .when(!enabled, |el| el.opacity(0.5))
        .focus_visible(move |el| el.border_color(c(style.accent)))
        .on_click(cx.listener(move |editor, event: &ClickEvent, window, cx| {
            if !editor.editable() {
                return;
            }
            if !event.is_keyboard() {
                window.focus(&focus, cx);
            }
            if event.is_keyboard() && editor.open_menu == Some(kind) {
                let ix = editor.menu_highlighted;
                if let Some(option) = options_for_click.get(ix) {
                    commit(editor, kind, option, cx);
                }
                return;
            }
            if editor.open_menu == Some(kind) || (kind == Menu::FontFamily && editor.font_menu_open)
            {
                close(editor, cx);
            } else if !editor.menu_dismissed_at.is_some_and(|(dismissed, at)| {
                dismissed == kind && at.elapsed() < Duration::from_millis(400)
            }) {
                open(editor, kind, selected_ix, window, cx);
            }
        }))
        .on_key_down({
            let options = visible
                .iter()
                .map(|item| OptionItem {
                    label: item.label.clone(),
                    value: item.value.clone(),
                    palette: item.palette,
                })
                .collect::<Vec<_>>();
            cx.listener(move |editor, event: &KeyDownEvent, window, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    return;
                }
                if key_down(
                    editor,
                    kind,
                    event.keystroke.key.as_str(),
                    selected_ix,
                    &options,
                    window,
                    cx,
                ) {
                    cx.stop_propagation();
                }
            })
        });
    if let Some(palette) = visible
        .iter()
        .find(|item| item.value == current_value)
        .and_then(|item| item.palette)
    {
        trigger = trigger.child(chip(palette));
    }
    trigger = trigger
        .child(div().flex_1().min_w_0().truncate().child(label))
        .child(
            div()
                .w(px(14.))
                .flex_none()
                .text_size(px(14.))
                .text_color(c(style.muted))
                .child("⌄"),
        );

    if is_open {
        let mut rows = div()
            .id(format!("{id}-list-{}", editor.menu_generation))
            .debug_selector(move || format!("{id}-list"))
            .max_h(px(if kind == Menu::FontFamily { 240. } else { 300. }))
            .overflow_y_scrollbar()
            .flex()
            .flex_col()
            .gap(px(2.));
        if visible.is_empty() {
            rows = rows.child(
                div()
                    .px(px(8.))
                    .py(px(6.))
                    .text_size(style.text_px(12.))
                    .text_color(c(style.muted))
                    .child("Шрифты не найдены"),
            );
        }
        for (ix, item) in visible.iter().enumerate() {
            let value = item.value.clone();
            let label = item.label.clone();
            let palette = item.palette;
            let active = item.value == current_value;
            let row_id = format!("{id}-option-{ix}");
            rows = rows.child(
                div()
                    .id(SharedString::from(row_id.clone()))
                    .debug_selector(move || row_id.clone())
                    .role(Role::MenuItemRadio)
                    .aria_label(label.clone())
                    .aria_toggled(if active {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(8.))
                    .text_size(px(13.))
                    .text_color(c(style.foreground))
                    .bg(if active {
                        rgba(style.foreground, 0.10)
                    } else if ix == highlighted {
                        rgba(style.foreground, 0.08)
                    } else {
                        transparent_black()
                    })
                    .hover(|el| el.bg(rgba(style.foreground, 0.08)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |editor, _, _, cx| {
                        cx.stop_propagation();
                        commit(
                            editor,
                            kind,
                            &OptionItem::new(label.clone(), value.clone()),
                            cx,
                        );
                    }))
                    .children(palette.map(chip))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(item.label.clone()),
                    )
                    .child(
                        div()
                            .w(px(18.))
                            .flex_none()
                            .text_size(px(14.))
                            .text_color(c(style.accent))
                            .child(if active { "✓" } else { "" }),
                    ),
            );
        }
        let mut menu = div()
            .w(px(menu_width))
            .max_w_full()
            .max_h(px(((f32::from(window.viewport_size().height) - 32.) / 2.
                - 6.)
                .clamp(1., 320.)))
            .rounded(px(12.))
            .border_1()
            .border_color(c(style.border))
            .shadow_lg()
            .bg(c(style.surface))
            .p(px(4.))
            .flex()
            .flex_col()
            .gap(px(4.))
            .on_mouse_down_out(cx.listener(move |editor, _, _, cx| close(editor, cx)))
            .on_key_down({
                let options = visible
                    .iter()
                    .map(|item| OptionItem {
                        label: item.label.clone(),
                        value: item.value.clone(),
                        palette: item.palette,
                    })
                    .collect::<Vec<_>>();
                cx.listener(move |editor, event: &KeyDownEvent, window, cx| {
                    if key_down(
                        editor,
                        kind,
                        event.keystroke.key.as_str(),
                        selected_ix,
                        &options,
                        window,
                        cx,
                    ) {
                        cx.stop_propagation();
                    }
                })
            });
        if let Some(heading) = heading {
            menu = menu.child(
                div()
                    .px(px(8.))
                    .pt(px(6.))
                    .pb(px(4.))
                    .text_size(style.text_px(10.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(c(style.muted))
                    .child(heading.to_uppercase()),
            );
        }
        if kind == Menu::FontFamily {
            let input = editor.font_search_input(window, cx);
            menu = menu.child(
                div().px(px(10.)).py(px(6.)).mb(px(4.)).child(
                    gpui_component::input::Input::new(&input)
                        .h(px(32.))
                        .text_size(px(12.5)),
                ),
            );
        }
        menu = menu.child(rows);
        trigger = trigger.child(
            div().absolute().bottom_0().right_0().size_0().child(
                deferred(
                    anchored()
                        .anchor(Anchor::TopRight)
                        .snap_to_window_with_margin(px(8.))
                        .child(div().occlude().pt(px(6.)).child(menu)),
                )
                .priority(1),
            ),
        );
    }
    trigger
}
