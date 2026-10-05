//! Shared appearance editor, faithfully ported from Cortex's
//! `manager-gpui/src/views/appearance.rs` (and its `colors`/`selector`/
//! `typography`/`previews` submodules). This crate owns only the visuals,
//! optimistic local state and keyboard/search behavior; callers own Engine
//! transport, persistence and app state entirely. A host embeds this as a
//! `gpui::Render` entity: call [`AppearanceEditor::ingest`] with the latest
//! server snapshot, [`AppearanceEditor::configure`] to set style/capability
//! toggles, and provide an `on_patch` callback to forward user edits.
use crate::settings::UiStyle;
use crate::theme::{c, rgba};
use crate::toggle::toggle;
use ::gpui::{prelude::*, *};
use gpui_component::input::InputState;
use gpui_component::Disableable;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

mod colors;
mod previews;
mod selector;
mod typography;

#[cfg(test)]
mod tests;

/// Engine-shaped settings snapshot. Field set and defaults mirror Cortex's
/// `appearance_state::Settings` exactly, since the host's `ingest`/patch
/// payloads are the same JSON shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct AppearanceSettings {
    schema_version: u32,
    mode: String,
    light_theme: String,
    dark_theme: String,
    accent_source: String,
    accent_color: Option<String>,
    follow_apps: bool,
    material: String,
    font_family: String,
    font_size: f32,
    revision: u64,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            mode: "dark".into(),
            light_theme: "default".into(),
            dark_theme: "default".into(),
            accent_source: "theme".into(),
            accent_color: None,
            follow_apps: false,
            material: "opaque".into(),
            font_family: "Inter".into(),
            font_size: 13.,
            revision: 0,
        }
    }
}

/// Transient selector state; the host snapshot remains the sole source of
/// accepted values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Menu {
    LightTheme,
    DarkTheme,
    FontFamily,
    FontSize,
    Material,
}

type OnPatch = Rc<dyn for<'a> Fn(Value, &mut Context<'a, AppearanceEditor>)>;

pub struct AppearanceEditor {
    on_patch: OnPatch,
    style: UiStyle,
    show_follow_apps: bool,
    editable: bool,
    status: Option<SharedString>,

    ready: bool,
    /// Last snapshot accepted from `ingest`; `reject` restores this.
    accepted: AppearanceSettings,
    /// Currently displayed snapshot, which may be ahead of `accepted` while
    /// a locally-applied edit has not yet echoed back through `ingest`.
    settings: AppearanceSettings,
    materials: Vec<String>,
    wallpaper_supported: bool,
    wallpaper_accent: Option<u32>,
    wallpaper_error: Option<String>,
    fonts: Vec<String>,

    font_menu_open: bool,
    open_menu: Option<Menu>,
    menu_highlighted: usize,
    menu_dismissed_at: Option<(Menu, Instant)>,
    menu_query: String,
    menu_generation: u64,
    tile_focus: HashMap<String, FocusHandle>,
    font_search: Option<Entity<InputState>>,
}

impl AppearanceEditor {
    pub fn new(
        cx: &mut Context<Self>,
        on_patch: impl Fn(Value, &mut Context<Self>) + 'static,
    ) -> Self {
        let mut fonts = cx.text_system().all_font_names();
        fonts.push(".SystemUIFont".into());
        fonts.sort_by_key(|name| name.to_lowercase());
        fonts.dedup();
        Self {
            on_patch: Rc::new(on_patch),
            style: UiStyle::default(),
            show_follow_apps: true,
            editable: true,
            status: None,
            ready: false,
            accepted: AppearanceSettings::default(),
            settings: AppearanceSettings::default(),
            materials: vec!["default".into(), "opaque".into()],
            wallpaper_supported: false,
            wallpaper_accent: None,
            wallpaper_error: None,
            fonts,
            font_menu_open: false,
            open_menu: None,
            menu_highlighted: 0,
            menu_dismissed_at: None,
            menu_query: String::new(),
            menu_generation: 0,
            tile_focus: HashMap::new(),
            font_search: None,
        }
    }

    /// Apply host-resolved style, whether the follow-apps row is offered at
    /// all, and whether controls accept input.
    pub fn configure(&mut self, style: UiStyle, show_follow_apps: bool, editable: bool) {
        self.style = style;
        self.show_follow_apps = show_follow_apps;
        self.editable = editable;
    }

    /// Optional status banner shown above the sections (e.g. a load/failure
    /// message). Pass an empty string to clear it.
    pub fn status(&mut self, status: impl Into<SharedString>) {
        let status = status.into();
        self.status = if status.is_empty() {
            None
        } else {
            Some(status)
        };
    }

    /// Ingest a host snapshot shaped like Cortex's `appearance.get`/`.set`
    /// response: `{"settings": {...}, "capabilities": {...}, ...}`. Returns
    /// `false` (and leaves state untouched) if the payload is malformed or
    /// carries an unsupported schema/font size, matching Cortex's guard.
    ///
    /// A response identical to the last accepted snapshot while a local edit
    /// is still outstanding (optimistic `settings != accepted`) is treated as
    /// a stale echo: capabilities still refresh, but the optimistic value is
    /// not clobbered. This lets a host re-deliver the same snapshot (e.g. on
    /// a duplicate poll) without undoing an in-flight user edit.
    pub fn ingest(&mut self, response: &Value) -> bool {
        let Ok(settings) =
            serde_json::from_value::<AppearanceSettings>(response["settings"].clone())
        else {
            return false;
        };
        if settings.schema_version != 1
            || !settings.font_size.is_finite()
            || !(11. ..=18.).contains(&settings.font_size)
        {
            return false;
        }
        let stale_echo = self.ready && settings == self.accepted && self.settings != self.accepted;
        if !stale_echo {
            self.accepted = settings.clone();
            self.settings = settings;
        }
        self.ready = true;
        self.materials = response["capabilities"]["materials"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_else(|| vec!["default".into(), "opaque".into()]);
        self.wallpaper_supported = response["capabilities"]["wallpaper_accent"]
            .as_bool()
            .unwrap_or(false);
        self.wallpaper_accent = response["wallpaper_accent"].as_str().and_then(parse_color);
        self.wallpaper_error = response["wallpaper_error"].as_str().map(str::to_owned);
        true
    }

    /// Discard any optimistic edit and restore the last accepted snapshot —
    /// for a host that learns a `patch` round-trip failed.
    pub fn reject(&mut self) {
        self.settings = self.accepted.clone();
    }

    fn editable(&self) -> bool {
        self.ready && self.editable
    }

    /// Apply a patch optimistically to the displayed snapshot, then forward
    /// it to the host via `on_patch`. No-ops while locked.
    fn patch(&mut self, params: Value, cx: &mut Context<Self>) {
        if !self.editable() {
            return;
        }
        if let Some(patch) = params.as_object() {
            if let Ok(mut value) = serde_json::to_value(&self.settings) {
                if let Some(settings) = value.as_object_mut() {
                    for (key, item) in patch {
                        settings.insert(key.clone(), item.clone());
                    }
                }
                if let Ok(next) = serde_json::from_value::<AppearanceSettings>(value) {
                    self.settings = next;
                }
            }
        }
        let on_patch = self.on_patch.clone();
        on_patch(params, cx);
        cx.notify();
    }

    fn font_search_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        self.font_search
            .get_or_insert_with(|| {
                cx.new(|cx| InputState::new(window, cx).placeholder("Поиск шрифта"))
            })
            .clone()
    }
}

fn parse_color(value: &str) -> Option<u32> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

fn loading_badge(style: UiStyle) -> Div {
    div()
        .px(px(8.))
        .py(px(2.))
        .rounded_full()
        .bg(rgba(style.muted, 0.15))
        .text_size(style.text_px(12.))
        .text_color(c(style.muted))
        .child("Загрузка…")
}

fn section_label(style: UiStyle, label: &'static str) -> Div {
    style.section_label(label)
}
fn section_block(style: UiStyle, label: &'static str, block: impl IntoElement) -> Div {
    style.section_block(label, block)
}
fn settings_card(style: UiStyle) -> Div {
    style.settings_card()
}
fn card_row(style: UiStyle, first: bool) -> Div {
    style.card_row(first)
}
fn row_title(style: UiStyle, title: &'static str) -> Div {
    style.row_title(title)
}
fn row_meta(style: UiStyle, text: &str) -> Div {
    style.row_meta(text)
}

impl Render for AppearanceEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = self.style;
        let enabled = self.editable();
        let wallpaper = self.ready && self.settings.accent_source == "wallpaper";
        let wallpaper_control = if self.ready {
            div()
                .flex_none()
                .debug_selector(|| "wallpaper-theme-colors".into())
                .child(
                    toggle(
                        "wallpaper-theme-colors",
                        wallpaper,
                        cx,
                        |this, enabled, cx| {
                            this.patch(
                                if enabled {
                                    json!({"accent_source":"wallpaper"})
                                } else {
                                    json!({"accent_source":"theme","accent_color":Value::Null})
                                },
                                cx,
                            );
                        },
                    )
                    .accessibility_label("Цвета обоев")
                    .disabled(!enabled || !self.wallpaper_supported),
                )
                .into_any_element()
        } else {
            loading_badge(style).into_any_element()
        };
        let follow = self.settings.follow_apps;
        let follow_control = if self.ready {
            div()
                .flex_none()
                .debug_selector(|| "appearance-follow-apps".into())
                .child(
                    toggle("appearance-follow-apps", follow, cx, |this, follow, cx| {
                        this.patch(json!({"follow_apps":follow}), cx);
                    })
                    .accessibility_label("Единый стиль приложений")
                    .disabled(!enabled),
                )
                .into_any_element()
        } else {
            loading_badge(style).into_any_element()
        };
        let material = self.settings.material.clone();
        let material_options = [
            ("default", "Цвета темы"),
            ("frosted", "Матовое стекло"),
            ("opaque", "Непрозрачный"),
            ("acrylic", "Acrylic"),
            ("mica", "Mica"),
        ]
        .into_iter()
        .filter(|(key, _)| self.materials.iter().any(|item| item == key))
        .map(|(key, label)| selector::OptionItem::new(label, key))
        .collect();
        let material_select = selector::select(
            self,
            selector::SelectSpec {
                kind: Menu::Material,
                id: "appearance-surface",
                aria_label: "Стекло",
                current: material.clone(),
                options: material_options,
                trigger_width: 148.,
                menu_width: 160.,
                heading: None,
            },
            window,
            cx,
        );
        let color_card = settings_card(style)
            .id("appearance-theme-card")
            .debug_selector(|| "appearance-theme-card".into())
            .children(previews::theme_rows(self, window, cx))
            .child(
                card_row(style, false)
                    .debug_selector(|| "appearance-accent-card".into())
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.))
                            .child(row_title(style, "Акцентный цвет"))
                            .child(row_meta(
                                style,
                                if wallpaper {
                                    "Цвета обоев включены; этот акцент используется, когда они выключены."
                                } else {
                                    "Цвет темы или один из образцов."
                                },
                            )),
                    )
                    .child(colors::accent_controls(self, cx)),
            )
            .child(
                card_row(style, false)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.))
                            .child(row_title(style, "Цвета обоев"))
                            .child(row_meta(style, colors::wallpaper_meta(self))),
                    )
                    .child(wallpaper_control),
            );
        let mut material_card = settings_card(style)
            .id("appearance-material-card")
            .debug_selector(|| "appearance-material-card".into())
            .child(
                card_row(style, true)
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.))
                            .child(row_title(style, "Стекло"))
                            .child(row_meta(
                                style,
                                match material.as_str() {
                                    "frosted" | "acrylic" | "mica" => "Прозрачные поверхности.",
                                    "opaque" => "Сплошные поверхности.",
                                    _ => "Цвета темы: сплошные поверхности.",
                                },
                            )),
                    )
                    .child(material_select),
            );
        if self.show_follow_apps {
            material_card = material_card.child(
                card_row(style, false)
                    .id("appearance-apps-card")
                    .debug_selector(|| "appearance-apps-card".into())
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(160.))
                            .child(row_title(style, "Единый стиль приложений"))
                            .child(row_meta(
                                style,
                                "Agenda использует этот стиль. Остальным нужен API Engine.",
                            )),
                    )
                    .child(follow_control),
            );
        }
        div()
            .w_full()
            .flex()
            .flex_col()
            .when_some(self.status.clone(), |page, status| {
                page.child(style.empty(&status))
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(section_label(style, "Цветовая схема"))
                    .child(previews::modes(self, cx)),
            )
            .child(color_card.mt(px(16.)))
            .child(section_block(style, "Материал", material_card))
            .child(typography::render(self, window, cx))
    }
}
