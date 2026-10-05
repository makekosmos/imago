use super::{AppearanceEditor, Menu};
use crate::settings::UiStyle;
use gpui::{Entity, KeyDownEvent, Keystroke, TestAppContext, VisualTestContext};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::rc::Rc;

fn sample(overrides: Value) -> Value {
    let mut settings = json!({
        "schema_version": 1,
        "mode": "dark",
        "light_theme": "default",
        "dark_theme": "default",
        "accent_source": "theme",
        "accent_color": null,
        "follow_apps": false,
        "material": "opaque",
        "font_family": "Inter",
        "font_size": 13.0,
        "revision": 0,
    });
    if let (Some(obj), Some(patch)) = (settings.as_object_mut(), overrides.as_object()) {
        for (key, item) in patch {
            obj.insert(key.clone(), item.clone());
        }
    }
    json!({
        "settings": settings,
        "capabilities": {
            "materials": ["default", "opaque", "frosted"],
            "wallpaper_accent": true,
        },
    })
}

fn launch(
    cx: &mut TestAppContext,
) -> (
    Entity<AppearanceEditor>,
    Rc<RefCell<Vec<Value>>>,
    &mut VisualTestContext,
) {
    cx.update(gpui_component::init);
    let log = Rc::new(RefCell::new(Vec::new()));
    let sink = log.clone();
    let (entity, cx) = cx.add_window_view(move |_, cx| {
        AppearanceEditor::new(cx, move |value, _| sink.borrow_mut().push(value))
    });
    (entity, log, cx)
}

#[gpui::test]
fn ingest_rejects_bad_schema_and_out_of_range_font_size(cx: &mut TestAppContext) {
    let (editor, _log, cx) = launch(cx);
    assert!(editor.update(cx, |editor, _| editor.ingest(&sample(json!({})))));
    assert!(!editor.update(cx, |editor, _| editor
        .ingest(&sample(json!({"schema_version": 2})))));
    assert!(!editor.update(cx, |editor, _| editor
        .ingest(&sample(json!({"font_size": 99.0})))));
    assert!(!editor.update(cx, |editor, _| editor
        .ingest(&json!({"settings": "nonsense"}))));
}

#[gpui::test]
fn duplicate_ingest_does_not_clobber_optimistic_edit(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, _| assert!(editor.ingest(&sample(json!({})))));
    editor.update(cx, |editor, cx| {
        editor.configure(UiStyle::default(), true, true);
        editor.patch(json!({"mode": "light"}), cx);
    });
    assert_eq!(
        editor.read_with(cx, |e, _| e.settings.mode.clone()),
        "light"
    );

    // Host re-delivers the pre-edit snapshot (e.g. a duplicate poll); the
    // optimistic edit must survive since it hasn't been echoed back yet.
    editor.update(cx, |editor, _| assert!(editor.ingest(&sample(json!({})))));
    assert_eq!(
        editor.read_with(cx, |e, _| e.settings.mode.clone()),
        "light"
    );
    assert_eq!(log.borrow().len(), 1);

    // Host catches up: the accepted snapshot now matches the optimistic one.
    editor.update(cx, |editor, _| {
        assert!(editor.ingest(&sample(json!({"mode": "light"}))))
    });
    assert_eq!(
        editor.read_with(cx, |e, _| e.accepted.mode.clone()),
        "light"
    );
    assert_eq!(
        editor.read_with(cx, |e, _| e.settings.mode.clone()),
        "light"
    );
}

#[gpui::test]
fn reject_restores_last_accepted_snapshot(cx: &mut TestAppContext) {
    let (editor, _log, cx) = launch(cx);
    editor.update(cx, |editor, _| assert!(editor.ingest(&sample(json!({})))));
    editor.update(cx, |editor, cx| {
        editor.configure(UiStyle::default(), true, true);
        editor.patch(json!({"mode": "light"}), cx);
    });
    assert_eq!(
        editor.read_with(cx, |e, _| e.settings.mode.clone()),
        "light"
    );
    editor.update(cx, |editor, _| editor.reject());
    assert_eq!(editor.read_with(cx, |e, _| e.settings.mode.clone()), "dark");
}

#[gpui::test]
fn writes_are_dropped_before_ready_and_when_locked(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    // Not ready yet (no ingest): patches are dropped and nothing is sent.
    editor.update(cx, |editor, cx| editor.patch(json!({"mode": "light"}), cx));
    assert!(log.borrow().is_empty());
    assert_eq!(editor.read_with(cx, |e, _| e.settings.mode.clone()), "dark");

    editor.update(cx, |editor, _| assert!(editor.ingest(&sample(json!({})))));
    editor.update(cx, |editor, _| {
        editor.configure(UiStyle::default(), true, false)
    });
    editor.update(cx, |editor, cx| editor.patch(json!({"mode": "light"}), cx));
    assert!(log.borrow().is_empty());
    assert_eq!(editor.read_with(cx, |e, _| e.settings.mode.clone()), "dark");
}

#[gpui::test]
fn show_follow_apps_false_hides_row_and_never_emits(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), false, true);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("appearance-apps-card").is_none());
    assert!(log.borrow().iter().all(|v| v.get("follow_apps").is_none()));

    editor.update(cx, |editor, cx| {
        editor.configure(UiStyle::default(), true, true);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("appearance-apps-card").is_some());
}

#[gpui::test]
fn editable_false_disables_controls_and_clicks_are_inert(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), true, false);
        cx.notify();
    });
    cx.run_until_parked();
    let tile = cx.debug_bounds("appearance-mode-light").unwrap();
    cx.simulate_click(tile.center(), Default::default());
    cx.run_until_parked();
    assert!(log.borrow().is_empty());
    assert_eq!(editor.read_with(cx, |e, _| e.settings.mode.clone()), "dark");
}

#[gpui::test]
fn sections_render_their_named_controls(cx: &mut TestAppContext) {
    let (editor, _log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), true, true);
        cx.notify();
    });
    cx.run_until_parked();
    for id in [
        "appearance-mode-system",
        "appearance-mode-light",
        "appearance-mode-dark",
        "appearance-theme-card",
        "light-theme-selector",
        "dark-theme-selector",
        "appearance-accent-card",
        "accent-theme",
        "accent-Zeron",
        "wallpaper-theme-colors",
        "appearance-material-card",
        "appearance-surface",
        "appearance-apps-card",
        "appearance-follow-apps",
        "appearance-font-card",
        "appearance-font-picker",
        "appearance-font-size-dropdown",
    ] {
        assert!(
            cx.debug_bounds(id).is_some(),
            "expected section control {id} to render"
        );
    }
}

#[gpui::test]
fn mode_tile_click_patches_mode(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), true, true);
        cx.notify();
    });
    cx.run_until_parked();
    let tile = cx.debug_bounds("appearance-mode-light").unwrap();
    cx.simulate_click(tile.center(), Default::default());
    cx.run_until_parked();
    assert_eq!(log.borrow().last().unwrap(), &json!({"mode": "light"}));
    assert_eq!(
        editor.read_with(cx, |e, _| e.settings.mode.clone()),
        "light"
    );
}

#[gpui::test]
fn accent_swatch_click_patches_custom_accent(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), true, true);
        cx.notify();
    });
    cx.run_until_parked();
    let swatch = cx.debug_bounds("accent-Zeron").unwrap();
    cx.simulate_click(swatch.center(), Default::default());
    cx.run_until_parked();
    let sent = log.borrow().last().cloned().unwrap();
    assert_eq!(sent["accent_source"], "custom");
    assert!(sent["accent_color"].is_string());
}

#[gpui::test]
fn material_selector_opens_and_commits_by_click(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), true, true);
        cx.notify();
    });
    cx.run_until_parked();
    let trigger = cx.debug_bounds("appearance-surface").unwrap();
    cx.simulate_click(trigger.center(), Default::default());
    cx.run_until_parked();
    assert_eq!(
        editor.read_with(cx, |e, _| e.open_menu),
        Some(Menu::Material)
    );
    let option = cx.debug_bounds("appearance-surface-option-0").unwrap();
    cx.simulate_click(option.center(), Default::default());
    cx.run_until_parked();
    assert_eq!(editor.read_with(cx, |e, _| e.open_menu), None);
    assert!(log.borrow().last().unwrap().get("material").is_some());
}

#[gpui::test]
fn theme_selector_keyboard_down_then_enter_commits(cx: &mut TestAppContext) {
    let (editor, log, cx) = launch(cx);
    editor.update(cx, |editor, cx| {
        assert!(editor.ingest(&sample(json!({}))));
        editor.configure(UiStyle::default(), true, true);
        cx.notify();
    });
    cx.run_until_parked();
    let trigger = cx.debug_bounds("light-theme-selector").unwrap();
    cx.simulate_click(trigger.center(), Default::default());
    cx.run_until_parked();
    assert_eq!(
        editor.read_with(cx, |e, _| e.open_menu),
        Some(Menu::LightTheme)
    );
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("down").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    let enter = Keystroke::parse("enter").unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: enter.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke: enter });
    cx.run_until_parked();
    assert_eq!(editor.read_with(cx, |e, _| e.open_menu), None);
    assert!(log.borrow().last().unwrap().get("light_theme").is_some());
}
