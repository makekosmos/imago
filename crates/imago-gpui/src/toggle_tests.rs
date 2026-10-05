use super::{surface_tones, toggle, Toggle, ToggleColors, Travel, HAPTICS, HEIGHT, WIDTH};
use gpui::{div, prelude::*, px, Context, IntoElement, Render, TestAppContext, Window};
use gpui_component::Disableable;
use std::time::Instant;

#[test]
fn travel_matches_zeron_and_reverses_without_jumping() {
    let start = Instant::now();
    let travel = Travel {
        from: 0.,
        target: 1.,
        started: start,
    };
    let halfway = start + std::time::Duration::from_millis(90);
    assert_eq!(travel.value(halfway), 0.875);
    assert_eq!(
        travel.value(start + std::time::Duration::from_millis(180)),
        1.
    );
    let reverse = Travel {
        from: travel.value(halfway),
        target: 0.,
        started: halfway,
    };
    assert_eq!(reverse.value(halfway), travel.value(halfway));
    assert_eq!(
        reverse.value(halfway + std::time::Duration::from_millis(180)),
        0.
    );
}

struct Harness {
    checked: bool,
    disabled: bool,
    changes: usize,
    builder: bool,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let switch = if self.builder {
            let listener = cx.listener(|this, next: &bool, _, cx| {
                this.checked = *next;
                this.changes += 1;
                cx.notify();
            });
            Toggle::new("test-toggle", self.checked)
                .on_change(move |next, window, cx| listener(&next, window, cx))
                .glass(true)
                .colors(ToggleColors {
                    accent: 0x548bd0,
                    foreground: 0xffffff,
                    surface: 0x202020,
                })
        } else {
            toggle("test-toggle", self.checked, cx, |this, next, _| {
                this.checked = next;
                this.changes += 1;
            })
        };
        div().child(
            div()
                .id("toggle-holder")
                .w(px(WIDTH))
                .debug_selector(|| "toggle-holder".into())
                .child(
                    switch
                        .disabled(self.disabled)
                        .accessibility_label("Тестовый переключатель"),
                ),
        )
    }
}

#[gpui::test]
fn shared_switch_preserves_geometry_accessibility_activation_and_haptics(cx: &mut TestAppContext) {
    check_activation(cx, false);
}

#[gpui::test]
fn builder_preserves_geometry_accessibility_activation_and_haptics(cx: &mut TestAppContext) {
    check_activation(cx, true);
}

fn check_activation(cx: &mut TestAppContext, builder: bool) {
    cx.update(gpui_component::init);
    cx.update(crate::theme::apply);
    let (harness, cx) = cx.add_window_view(|_, _| Harness {
        checked: false,
        disabled: false,
        changes: 0,
        builder,
    });
    let bounds = cx.debug_bounds("toggle-holder").unwrap();
    assert!((f32::from(bounds.size.width) - WIDTH).abs() < 1.);
    assert!((f32::from(bounds.size.height) - HEIGHT).abs() < 1.);
    let before = HAPTICS.with(|count| count.get());
    cx.simulate_click(bounds.center(), Default::default());
    assert!(harness.read_with(cx, |h, _| h.checked));
    assert_eq!(HAPTICS.with(|count| count.get()) - before, 1);
    let tree = cx
        .update(|window, _| window.debug_a11y_tree_json())
        .unwrap();
    assert!(tree.contains("Тестовый переключатель"));
    assert!(tree.contains("Switch"));
    cx.update(|_, cx| cx.refresh_windows());
    let keystroke = gpui::Keystroke::parse("space").unwrap();
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    assert!(!harness.read_with(cx, |h, _| h.checked));
    assert_eq!(HAPTICS.with(|count| count.get()) - before, 2);
    harness.update(cx, |h, cx| {
        h.disabled = true;
        cx.notify();
    });
    cx.update(|_, cx| cx.refresh_windows());
    cx.simulate_click(bounds.center(), Default::default());
    assert_eq!(harness.read_with(cx, |h, _| h.changes), 2);
    assert_eq!(HAPTICS.with(|count| count.get()) - before, 2);
    let keystroke = gpui::Keystroke::parse("space").unwrap();
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    assert!(!harness.read_with(cx, |h, _| h.checked));
    assert_eq!(harness.read_with(cx, |h, _| h.changes), 2);
    assert_eq!(HAPTICS.with(|count| count.get()) - before, 2);
}

#[test]
fn glass_is_opt_in_and_palette_override_is_explicit() {
    let switch = Toggle::new("defaults", false);
    assert!(!switch.glass);
    assert!(!switch.disabled);
    assert!(switch.colors.is_none());
    let colors = ToggleColors {
        accent: 0x548bd0,
        foreground: 0xffffff,
        surface: 0x202020,
    };
    let switch = switch.glass(true).colors(colors);
    assert!(switch.glass);
    assert_eq!(switch.colors, Some(colors));
    let base = crate::theme::c(colors.surface);
    for thumb in [false, true] {
        assert_eq!(surface_tones(base, thumb, false), (base, base));
        let (light, shade) = surface_tones(base, thumb, true);
        let (light_opacity, shade_opacity) = if thumb { (0.12, 0.07) } else { (0.07, 0.09) };
        assert_eq!(light, base.blend(gpui::white().opacity(light_opacity)));
        assert_eq!(shade, base.blend(gpui::black().opacity(shade_opacity)));
    }
}
