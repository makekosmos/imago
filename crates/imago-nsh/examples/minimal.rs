use gpui::{px, size, App, AppContext, WindowBounds, WindowOptions};
use imago_nsh::{mount_app_view, Bundle, NshApp};
use std::time::Duration;

fn main() {
    let log = std::env::var("IMAGO_NSH_EXAMPLE_LOG").unwrap_or_else(|_| "example.log".into());
    std::fs::write(&log, "application started").expect("write example status");
    gpui::application().run(|cx: &mut App| {
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(gpui::bounds(
                        gpui::point(px(-20_000.0), px(-20_000.0)),
                        size(px(900.0), px(600.0)),
                    ))),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| {
                        mount_app_view(
                            window,
                            cx,
                            NshApp::Js(Bundle::source(
                                include_str!("minimal/ui/.nsh/out/bundle.js").into(),
                            )),
                            |_| {},
                        )
                    })
                },
            )
            .expect("open offscreen example window")
            .into();
        std::fs::write(&log, "window opened").expect("write example status");
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            std::fs::write(&log, "capture scheduled").expect("write example status");
            let result = cx.update_window(window, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                window.render_to_image()
            });
            let out =
                std::env::var("IMAGO_NSH_EXAMPLE_PNG").unwrap_or_else(|_| "example.png".into());
            match result {
                Ok(Ok(image)) => {
                    image.save(out).expect("save offscreen example screenshot");
                    std::fs::write(&log, "screenshot saved").expect("write example status");
                }
                Ok(Err(error)) => {
                    std::fs::write(&log, error.to_string()).expect("write example error")
                }
                Err(error) => std::fs::write(&log, error.to_string()).expect("write example error"),
            }
            cx.update(|cx| cx.quit());
        })
        .detach();
        cx.activate(true);
    });
}
