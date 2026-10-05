//! One interface font row, ported from Cortex's `appearance::typography`,
//! using the shared selector geometry.
use super::{
    card_row, row_meta, row_title, section_block, selector, settings_card, AppearanceEditor, Menu,
};
use ::gpui::{prelude::*, *};

pub(super) fn render(
    editor: &mut AppearanceEditor,
    window: &mut Window,
    cx: &mut Context<AppearanceEditor>,
) -> Div {
    let style = editor.style;
    let family = editor.settings.font_family.clone();
    let mut options: Vec<_> = editor
        .fonts
        .iter()
        .map(|font| {
            selector::OptionItem::new(
                if font == ".SystemUIFont" {
                    "Системный"
                } else {
                    font
                },
                font,
            )
        })
        .collect();
    options.sort_by_key(|item| item.label.to_lowercase());
    let font = selector::select(
        editor,
        selector::SelectSpec {
            kind: Menu::FontFamily,
            id: "appearance-font-picker",
            aria_label: "Шрифт интерфейса",
            current: family.clone(),
            options,
            trigger_width: 220.,
            menu_width: 220.,
            heading: None,
        },
        window,
        cx,
    );
    // The host accepts 11..=18 and defaults to 13. Zeron's 20px rung is
    // unavailable in this protocol; 12.5 remains selectable for saved values.
    let sizes = [11., 12., 12.5, 13., 14., 15., 16., 18.]
        .into_iter()
        .map(|size| selector::OptionItem::new(format!("{size} px"), format!("{size}")))
        .collect();
    let size = selector::select(
        editor,
        selector::SelectSpec {
            kind: Menu::FontSize,
            id: "appearance-font-size-dropdown",
            aria_label: "Размер шрифта",
            current: format!("{}", editor.settings.font_size),
            options: sizes,
            trigger_width: 128.,
            menu_width: 128.,
            heading: None,
        },
        window,
        cx,
    );
    let mut block = section_block(
        style,
        "Шрифты",
        settings_card(style)
            .id("appearance-font-card")
            .debug_selector(|| "appearance-font-card".into())
            .child(
                card_row(style, true)
                    .child(
                        div()
                            .min_w(px(160.))
                            .flex_1()
                            .child(row_title(style, "Шрифт интерфейса"))
                            .child(row_meta(style, "Меню и текст интерфейса.")),
                    )
                    .child(
                        div()
                            .flex_none()
                            .max_w_full()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(8.))
                            .child(font)
                            .child(size),
                    ),
            ),
    );
    if editor.ready && !editor.fonts.contains(&family) {
        block = block.child(style.empty("Выбранный шрифт недоступен; используется системный."));
    }
    block
}
