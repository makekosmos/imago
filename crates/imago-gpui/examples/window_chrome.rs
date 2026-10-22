use gpui::{
    div, prelude::*, px, size, App, Bounds, Render, TitlebarOptions, Window, WindowBounds,
    WindowOptions,
};
use imago_gpui::chrome;

struct ChromeDemo;

impl Render for ChromeDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(gpui::rgb(0x17191d))
            .child(
                chrome::titlebar()
                    .bg(gpui::rgb(0x202329))
                    .child(
                        div()
                            .id("demo-title-drag")
                            .flex_1()
                            .h_full()
                            .flex()
                            .items_center()
                            .pl(px(16.))
                            .text_color(gpui::rgb(0xe7e9ee))
                            .child("Imago · gpui-fast window chrome")
                            .window_control_area(gpui::WindowControlArea::Drag),
                    )
                    .child(chrome::window_controls()),
            )
            .child(
                div()
                    .flex_1()
                    .p(px(24.))
                    .text_color(gpui::rgb(0xe7e9ee))
                    .child("Drag the title bar; test minimize, maximize, and close."),
            )
    }
}

fn main() {
    gpui::application()
        .with_assets(imago_gpui::assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            imago_gpui::theme::apply(cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(720.), px(420.)),
                        cx,
                    ))),
                    titlebar: Some(TitlebarOptions {
                        title: None,
                        appears_transparent: true,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| ChromeDemo),
            )
            .expect("open Imago window chrome demo");
            cx.activate(true);
        });
}
