//! Accessibility contract tests for shared imago-gpui chrome primitives.
//!
//! Each element is drawn inside a test window and probed through
//! [`gpui::Element::a11y_role`] / [`gpui::Element::write_a11y_info`] — the
//! same data `Window::debug_a11y_tree_json` exposes per node for whole
//! screens in the application repos.
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gpui::{accesskit, canvas, div, prelude::*, Context, Render, TestAppContext, Window};
use imago_gpui::{button, chrome};

/// `(role, node)` pairs the probe element records during prepaint.
type Captured = Arc<Mutex<Vec<(Option<gpui::Role>, accesskit::Node)>>>;
/// Element probe: builds a control and snapshots its emitted a11y facts.
type Build = Rc<dyn Fn(&mut Window, &mut gpui::App) -> (Option<gpui::Role>, accesskit::Node)>;

/// Renders a canvas whose prepaint callback runs the registered element
/// builders and records each element's emitted role + accesskit node.
struct Probe {
    builds: Vec<Build>,
    captured: Captured,
}

/// Snapshot `el`'s emitted accessibility facts: its role plus every property
/// `write_a11y_info` writes onto the accesskit node.
fn capture<E: gpui::Element>(el: E) -> (Option<gpui::Role>, accesskit::Node) {
    let role = el.a11y_role();
    let mut node = accesskit::Node::new(role.unwrap_or(accesskit::Role::GenericContainer));
    el.write_a11y_info(&mut node);
    (role, node)
}

/// Snapshot the accessibility facts a `gpui-component` [`Button`] emits.
///
/// `Button::render` returns another component (`gpui_base::Button`) whose own
/// render applies the role/name onto an inner `Stateful<Div>`, so the role
/// only exists once the component is laid out. We drive `request_layout` by
/// hand inside the canvas prepaint callback (a phase where GPUI explicitly
/// permits layout requests), recover the rendered inner element via
/// [`gpui::AnyElement::downcast_mut`], and probe that.
fn capture_button(
    button: gpui_component::button::Button,
    window: &mut Window,
    cx: &mut gpui::App,
) -> (Option<gpui::Role>, accesskit::Node) {
    let mut any = RenderOnce::render(button, window, cx).into_any_element();
    let view = any
        .downcast_mut::<gpui::ViewElement<gpui_base::Button>>()
        .expect("gpui-component Button renders to gpui_base::Button");
    let (_layout_id, request_layout) = view.request_layout(None, None, window, cx);
    let mut inner: gpui::AnyElement = request_layout.unwrap();
    let div = inner
        .downcast_mut::<gpui_base::ObservedElement<gpui::Stateful<gpui::Div>>>()
        .expect("gpui-component Button should render to a Stateful<Div>");
    let role = div.a11y_role();
    let mut node = accesskit::Node::new(role.unwrap_or(accesskit::Role::GenericContainer));
    div.write_a11y_info(&mut node);
    (role, node)
}

impl Render for Probe {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let captured = self.captured.clone();
        let builds = self.builds.clone();
        div().child(canvas(
            move |_, window, cx| {
                let mut out = captured.lock().unwrap();
                out.clear();
                for build in builds.iter() {
                    out.push(build(window, cx));
                }
            },
            |_, _, _, _| {},
        ))
    }
}

/// Open a test window hosting `probe` and draw one frame so the canvas
/// prepaint callback runs; returns the captured (role, node) list.
fn drawn(cx: &mut TestAppContext, probe: Probe) -> Vec<(Option<gpui::Role>, accesskit::Node)> {
    cx.update(gpui_component::init);
    let captured = probe.captured.clone();
    let (_, cx) = cx.add_window_view(move |_, _| probe);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let nodes = std::mem::take(&mut *captured.lock().unwrap());
    nodes
}

#[gpui::test]
fn sidebar_item_emits_button_role_and_label(cx: &mut TestAppContext) {
    let captured: Captured = Default::default();
    let nodes = drawn(
        cx,
        Probe {
            builds: vec![
                Rc::new(|_, _| {
                    capture(
                        chrome::sidebar_item(
                            "nav-inbox",
                            gpui_component::Icon::default(),
                            "Входящие",
                            true,
                        )
                        .into_element(),
                    )
                }),
                Rc::new(|_, _| {
                    capture(
                        chrome::sidebar_item(
                            "nav-settings",
                            gpui_component::Icon::default(),
                            "Настройки",
                            false,
                        )
                        .accessibility_label("Открыть настройки")
                        .into_element(),
                    )
                }),
            ],
            captured: captured.clone(),
        },
    );

    let (role, node) = &nodes[0];
    assert_eq!(*role, Some(gpui::Role::Button));
    // The visible label is the accessible name — callers don't repeat it.
    assert_eq!(node.label(), Some("Входящие"));
    assert_eq!(node.is_selected(), Some(true));
    // The element id rides along as a stable locator (AutomationId etc.).
    assert_eq!(node.author_id(), Some("nav-inbox"));

    let (role, node) = &nodes[1];
    assert_eq!(*role, Some(gpui::Role::Button));
    assert_eq!(node.label(), Some("Открыть настройки"));
    assert_eq!(node.is_selected(), Some(false));
}

#[gpui::test]
fn buttons_emit_button_role_and_label(cx: &mut TestAppContext) {
    let captured: Captured = Default::default();
    let nodes = drawn(
        cx,
        Probe {
            builds: vec![
                Rc::new(|window, cx| {
                    capture_button(button::primary("save").label("Сохранить"), window, cx)
                }),
                Rc::new(|window, cx| {
                    capture_button(
                        button::ghost("close").accessibility_label("Закрыть"),
                        window,
                        cx,
                    )
                }),
            ],
            captured: captured.clone(),
        },
    );

    let (role, node) = &nodes[0];
    assert_eq!(*role, Some(gpui::Role::Button));
    // gpui-component derives the name from `.label(...)`.
    assert_eq!(node.label(), Some("Сохранить"));

    let (role, node) = &nodes[1];
    assert_eq!(*role, Some(gpui::Role::Button));
    // Icon-only buttons take an explicit accessible name.
    assert_eq!(node.label(), Some("Закрыть"));
}

#[gpui::test]
fn window_caption_button_emits_role_and_russian_name(cx: &mut TestAppContext) {
    let captured: Captured = Default::default();
    let nodes = drawn(
        cx,
        Probe {
            builds: vec![Rc::new(|_, _| {
                capture(chrome::caption_button(
                    "win-close",
                    "icons/status-x.svg",
                    gpui::WindowControlArea::Close,
                    true,
                    "Закрыть",
                ))
            })],
            captured: captured.clone(),
        },
    );

    let (role, node) = &nodes[0];
    assert_eq!(*role, Some(gpui::Role::Button));
    assert_eq!(node.label(), Some("Закрыть"));
    assert_eq!(node.author_id(), Some("win-close"));
}
