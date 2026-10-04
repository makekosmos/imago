//! View primitives + JSON accessors: section/card/row/kv/badge/btn/toggle/
//! empty/slot_or and the serde_json::Value getters every view uses.
//!
//! Entity-bound helpers (`btn`, `toggle`, `slot_or`) are generic over the
//! root entity type so any GPUI app can use them; `slot_or` reads through
//! the [`Slots`] trait the app implements.
use ::gpui::{prelude::*, *};
use gpui_component::button::Button;
use gpui_component::switch::Switch;
use imago_gpui::button::{self, ButtonKind};
use serde_json::Value;

use crate::theme::*;

// --- Async data slots -------------------------------------------------------

/// Named async view-data slot: views stay total over missing data — Loading
/// and Failed render inline instead of gating the view.
pub enum Slot {
    Loading,
    Ready(Value),
    Failed(String),
}

/// Root entity exposing named [`Slot`]s (ManagerApp, the unified Cortex app,
/// agenda app, ...). Implement to make [`slot_or`] work on the entity.
pub trait Slots {
    fn slot(&self, key: &str) -> Option<&Slot>;
}

// --- JSON accessors ---------------------------------------------------------

pub fn vget<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or(&Value::Null)
}

pub fn vstr(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => s.clone(),
        // Numbers/bools/objects stringify; null renders like a missing key
        // rather than the literal "null".
        Some(other) if !other.is_null() => other.to_string(),
        _ => String::new(),
    }
}

pub fn vopt(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

pub fn vbool(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub fn vnum(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

pub fn varr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub fn fmt_bytes(n: f64) -> String {
    // Engine JSON can carry negative/NaN values for unknown sizes — clamp
    // to 0 rather than render "-512 Б"/"NaN Б".
    let n = if n.is_finite() && n > 0.0 { n } else { 0.0 };
    if n >= 1_073_741_824.0 {
        format!("{:.1} ГБ", n / 1_073_741_824.0)
    } else if n >= 1_048_576.0 {
        format!("{:.1} МБ", n / 1_048_576.0)
    } else if n >= 1024.0 {
        format!("{:.0} КБ", n / 1024.0)
    } else {
        format!("{:.0} Б", n)
    }
}

pub fn fmt_ms(ms: f64) -> String {
    let secs = ms / 1000.0;
    let days = (secs / 86400.0).floor();
    // Truncate like `days` — "{:.0}" rounds, so fractional 59.6 minutes
    // would otherwise render as "60 мин. назад".
    let h = ((secs % 86400.0) / 3600.0).floor();
    let m = ((secs % 3600.0) / 60.0).floor();
    if days >= 1.0 {
        format!("{days:.0} дн. назад")
    } else if h >= 1.0 {
        format!("{h:.0} ч. назад")
    } else if m >= 1.0 {
        format!("{m:.0} мин. назад")
    } else {
        "только что".into()
    }
}

// --- View primitives --------------------------------------------------------

pub fn section(title: &str, hint: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(
            div()
                .text_size(px(15.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(c(MUTED_FG()))
                .child(hint.to_string()),
        )
}

pub fn card() -> Div {
    div()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(c(BORDER()))
        .bg(c(CARD()))
        .p_4()
        .flex()
        .flex_col()
        .gap_2()
}

pub fn row(label: impl Into<String>, sub: impl Into<String>) -> Div {
    div()
        .w_full()
        .min_h_10()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(div().text_size(px(13.)).child(label.into()))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(c(MUTED_FG()))
                        .child(sub.into()),
                ),
        )
}

pub fn kv(key: &str, value: impl Into<String>) -> Div {
    div()
        .flex()
        .items_baseline()
        .gap_2()
        .child(
            div()
                .w(px(180.))
                .flex_none()
                .text_size(px(13.))
                .text_color(c(MUTED_FG()))
                .child(key.to_string()),
        )
        .child(div().flex_1().text_size(px(13.)).child(value.into()))
}

pub fn badge(text: impl Into<String>, color: u32) -> impl IntoElement {
    div()
        .px_2()
        .py_0p5()
        .rounded_full()
        .bg(fade(color, 0.15))
        .text_size(px(12.))
        .text_color(c(color))
        .child(text.into())
}

pub fn btn<T: 'static>(
    id: &'static str,
    label: &'static str,
    primary: bool,
    cx: &mut Context<T>,
    on_click: impl Fn(&mut T, &mut Context<T>) + 'static,
) -> Button {
    let kind = if primary {
        ButtonKind::Primary
    } else {
        ButtonKind::Ghost
    };
    button::button(id, kind)
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            on_click(this, cx);
            cx.notify();
        }))
}

/// Button whose id must be dynamic (per-row actions like disconnect/uninstall).
/// Pass `cx.listener(...)` — it adapts the entity handler to the App callback.
pub fn btn_id(
    id: &str,
    label: &'static str,
    listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Button {
    button::ghost(SharedString::from(id.to_string()))
        .label(label)
        .on_click(listener)
}

pub fn toggle<T: 'static>(
    id: &'static str,
    checked: bool,
    cx: &mut Context<T>,
    on_click: impl Fn(&mut T, bool, &mut Context<T>) + 'static,
) -> Switch {
    Switch::new(id)
        .checked(checked)
        .on_click(cx.listener(move |this, checked, _, cx| {
            on_click(this, *checked, cx);
            cx.notify();
        }))
}

pub fn empty(text: &str) -> impl IntoElement {
    div()
        .p_6()
        .text_size(px(13.))
        .text_color(c(MUTED_FG()))
        .child(text.to_string())
}

pub fn slot_or<A, F>(app: &A, slot: &str, render: F) -> AnyElement
where
    A: Slots + ?Sized,
    F: FnOnce(&Value) -> AnyElement,
{
    match app.slot(slot) {
        Some(Slot::Ready(v)) => render(v),
        Some(Slot::Failed(e)) => div()
            .text_size(px(13.))
            .text_color(c(DESTRUCTIVE()))
            .child(e.clone())
            .into_any_element(),
        _ => div()
            .text_size(px(13.))
            .text_color(c(MUTED_FG()))
            .child("Загрузка…")
            .into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::{fmt_bytes, fmt_ms, vstr};
    use serde_json::json;

    /// Byte counts come from Engine JSON — a negative or NaN value must
    /// not render "-512 Б"/"NaN Б" on screen; absent data shows "0 Б".
    #[test]
    fn fmt_bytes_sanitizes_negative_and_nan() {
        assert_eq!(fmt_bytes(-512.0), "0 Б");
        assert_eq!(fmt_bytes(f64::NAN), "0 Б");
        assert_eq!(fmt_bytes(f64::NEG_INFINITY), "0 Б");
        assert_eq!(fmt_bytes(0.0), "0 Б");
        assert_eq!(fmt_bytes(1536.0), "2 КБ");
    }

    /// JSON `null` must render like a missing key ("") — not the literal
    /// string "null" in the UI.
    #[test]
    fn vstr_renders_null_like_a_missing_key() {
        let value = json!({"name": null, "count": 42, "title": "Отчёт", "on": true});
        assert_eq!(vstr(&value, "missing"), "");
        assert_eq!(vstr(&value, "name"), "");
        assert_eq!(vstr(&value, "count"), "42");
        assert_eq!(vstr(&value, "title"), "Отчёт");
        assert_eq!(vstr(&value, "on"), "true");
    }

    /// "Ago" formatting truncates fractional units — 59.6 minutes is
    /// "59 мин. назад", never "60 мин. назад" (same for hours/days).
    #[test]
    fn fmt_ms_truncates_fractional_units() {
        assert_eq!(fmt_ms(59.6 * 60_000.0), "59 мин. назад");
        assert_eq!(fmt_ms(23.6 * 3_600_000.0), "23 ч. назад");
        assert_eq!(fmt_ms(1.5 * 86_400_000.0), "1 дн. назад");
        assert_eq!(fmt_ms(30_000.0), "только что");
    }
}
