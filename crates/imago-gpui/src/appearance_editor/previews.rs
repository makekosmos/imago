//! Mode tiles and theme rows, ported from Cortex's `appearance::previews`,
//! reusing this crate's own shared palette miniatures and mode tile.
use super::{card_row, row_title, selector, AppearanceEditor, Menu};
use crate::appearance::{miniature, system_preview, ModeTile};
use crate::palettes::THEMES;
use ::gpui::{prelude::*, *};
use gpui_component::Disableable;
use serde_json::json;

fn mode_card(
    editor: &mut AppearanceEditor,
    mode: &'static str,
    label: &'static str,
    icon: &'static str,
    preview: Div,
    cx: &mut Context<AppearanceEditor>,
) -> ModeTile {
    let id = format!("appearance-mode-{mode}");
    let focus = editor
        .tile_focus
        .entry(id.clone())
        .or_insert_with(|| cx.focus_handle())
        .clone();
    let selected = editor.ready && editor.settings.mode == mode;
    let weak = cx.weak_entity();
    ModeTile::new(SharedString::from(id), label, preview, focus)
        .icon(icon)
        .selected(selected)
        .disabled(!editor.editable())
        .style(editor.style)
        .on_select(move |_, cx| {
            let _ = weak.update(cx, |editor, cx| {
                if editor.editable() {
                    editor.patch(json!({"mode": mode}), cx);
                }
            });
        })
}

pub(super) fn modes(editor: &mut AppearanceEditor, cx: &mut Context<AppearanceEditor>) -> Div {
    let light = THEMES
        .iter()
        .find(|t| t.key == editor.settings.light_theme)
        .unwrap_or(&THEMES[0]);
    let dark = THEMES
        .iter()
        .find(|t| t.key == editor.settings.dark_theme)
        .unwrap_or(&THEMES[0]);
    div()
        .w_full()
        .flex()
        .items_start()
        .gap(px(16.))
        .child(mode_card(
            editor,
            "system",
            "Системная",
            "icons/monitor.svg",
            system_preview(&light.light, &dark.dark),
            cx,
        ))
        .child(mode_card(
            editor,
            "light",
            "Светлая",
            "icons/sun.svg",
            miniature(&light.light),
            cx,
        ))
        .child(mode_card(
            editor,
            "dark",
            "Тёмная",
            "icons/moon.svg",
            miniature(&dark.dark),
            cx,
        ))
}

pub(super) fn theme_rows(
    editor: &mut AppearanceEditor,
    window: &mut Window,
    cx: &mut Context<AppearanceEditor>,
) -> Vec<AnyElement> {
    let style = editor.style;
    let mut rows = Vec::new();
    for (ix, (kind, title, field, dark)) in [
        (Menu::LightTheme, "Светлая тема", "light_theme", false),
        (Menu::DarkTheme, "Тёмная тема", "dark_theme", true),
    ]
    .into_iter()
    .enumerate()
    {
        let selected = if dark {
            editor.settings.dark_theme.clone()
        } else {
            editor.settings.light_theme.clone()
        };
        let options = THEMES
            .iter()
            .map(|theme| {
                let palette = if dark { &theme.dark } else { &theme.light };
                selector::OptionItem::new(theme.name, theme.key).palette(
                    palette.card,
                    palette.bg,
                    palette.accent,
                    palette.border,
                )
            })
            .collect();
        let select = selector::select(
            editor,
            selector::SelectSpec {
                kind,
                id: if dark {
                    "dark-theme-selector"
                } else {
                    "light-theme-selector"
                },
                aria_label: title,
                current: selected,
                options,
                trigger_width: 218.,
                menu_width: 260.,
                heading: Some(if dark {
                    "Тёмные темы"
                } else {
                    "Светлые темы"
                }),
            },
            window,
            cx,
        );
        rows.push(
            card_row(style, ix == 0)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(160.))
                        .child(row_title(style, title)),
                )
                .child(select)
                .debug_selector(move || format!("appearance-{field}-row"))
                .into_any_element(),
        );
    }
    rows
}
