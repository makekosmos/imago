//! Shared UI contracts, independent of application or Engine state.
use crate::{
    about::{AboutHero, DiagnosticCheck, DiagnosticState, Diagnostics},
    appearance::{miniature, ModeTile},
    settings::{UiStyle, PAGE_WIDTH},
};
use gpui::{
    div, prelude::*, px, Context, Entity, FocusHandle, IntoElement, KeyDownEvent, Keystroke,
    Modifiers, Render, Styled, TestAppContext, Window,
};
use gpui_component::Disableable;

struct Harness {
    focus: FocusHandle,
    disabled: bool,
    selected: bool,
    activations: usize,
    expanded: bool,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let weak = cx.weak_entity();
        let diag_weak = weak.clone();
        div()
            .w(px(PAGE_WIDTH))
            .flex()
            .flex_col()
            .child(
                AboutHero::new("about-hero", "Example", "Версия 1.0")
                    .copyright("Example copyright"),
            )
            .child(
                ModeTile::new(
                    "appearance-mode-test",
                    "Светлая",
                    miniature(&crate::THEMES[0].light),
                    self.focus.clone(),
                )
                .selected(self.selected)
                .disabled(self.disabled)
                .style(UiStyle::default())
                .on_select(move |_, cx| {
                    weak.update(cx, |app, cx| {
                        app.activations += 1;
                        app.selected = true;
                        cx.notify();
                    })
                    .unwrap();
                }),
            )
            .child(
                Diagnostics::new(
                    "about-diagnostics-card",
                    vec![DiagnosticCheck::new("Engine", DiagnosticState::Fail)],
                )
                .expanded(self.expanded)
                .on_toggle(move |_, cx| {
                    diag_weak
                        .update(cx, |app, cx| {
                            app.expanded = !app.expanded;
                            cx.notify();
                        })
                        .unwrap();
                }),
            )
    }
}

fn launch(cx: &mut TestAppContext) -> (Entity<Harness>, &mut gpui::VisualTestContext) {
    cx.update(gpui_component::init);
    cx.add_window_view(|_, cx| Harness {
        focus: cx.focus_handle(),
        disabled: false,
        selected: false,
        activations: 0,
        expanded: false,
    })
}

#[gpui::test]
fn mode_tile_keyboard_activation_and_disabled_guard(cx: &mut TestAppContext) {
    let (app, cx) = launch(cx);
    cx.update(|window, cx| {
        let focus = app.read(cx).focus.clone();
        window.focus(&focus, cx);
    });
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers::default(),
            key: "space".into(),
            key_char: Some(" ".into()),
        },
        is_held: false,
        prefer_character_input: false,
    });
    assert_eq!(app.read_with(cx, |app, _| app.activations), 1);
    app.update(cx, |app, cx| {
        app.disabled = true;
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers::default(),
            key: "enter".into(),
            key_char: None,
        },
        is_held: false,
        prefer_character_input: false,
    });
    assert_eq!(app.read_with(cx, |app, _| app.activations), 1);
}

#[gpui::test]
fn diagnostics_expands_by_pointer_and_collapses_by_keyboard(cx: &mut TestAppContext) {
    let (app, cx) = launch(cx);
    let toggle = cx.debug_bounds("about-diagnostics-toggle").unwrap();
    cx.simulate_click(toggle.center(), Default::default());
    assert!(app.read_with(cx, |app, _| app.expanded));
    cx.update(|_, cx| cx.refresh_windows());
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
    assert!(!app.read_with(cx, |app, _| app.expanded));
}

#[gpui::test]
fn about_and_appearance_expose_named_semantics(cx: &mut TestAppContext) {
    let (_, cx) = launch(cx);
    cx.update(|_, cx| cx.refresh_windows());
    cx.run_until_parked();
    let tree = cx
        .update(|window, _| window.debug_a11y_tree_json())
        .expect("patched TestWindow activates a11y");
    assert!(tree.contains("Example"));
    assert!(tree.contains("Heading"));
    assert!(tree.contains("Режим темы: Светлая"));
    assert!(tree.contains("Список проверок диагностики"));
    assert_eq!(
        cx.debug_bounds("appearance-mode-test").unwrap().size.width,
        px(PAGE_WIDTH)
    );
    assert!(cx.debug_bounds("about-diagnostics-list").is_none());
}
