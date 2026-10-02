//! Shared chrome: frameless window controls and the field/view primitives
//! (re-exported here). App-shell pieces (sidebar, titlebar, banner) stay in
//! the consuming app — they bind to its root entity and nav types; shell
//! primitives come from `imago_gpui::chrome`.
use ::gpui::{prelude::*, *};

pub use crate::fields::*;

use crate::theme::*;

// --- Chrome -----------------------------------------------------------------

pub fn render_window_controls() -> impl IntoElement {
    const CAPTION_W: f32 = 46.0;
    const CLOSE_RED: u32 = 0xe81123;
    let specs = [
        ("min", "icons/window-min.svg", WindowControlArea::Min, false),
        (
            "close",
            "icons/status-x.svg",
            WindowControlArea::Close,
            true,
        ),
    ];
    let mut row = div().flex().flex_none().h_full();
    for (id, path, area, danger) in specs {
        row = row.child(
            div()
                .id(SharedString::from(format!("win-{id}")))
                .w(px(CAPTION_W))
                .h_full()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .text_color(fade(FG(), 0.82))
                .hover(move |button| {
                    if danger {
                        button.bg(c(CLOSE_RED)).text_color(c(0xffffff))
                    } else {
                        button.bg(fade(FG(), 0.10))
                    }
                })
                .child(svg().path(path).size(px(14.)).text_color(fade(FG(), 0.82)))
                .window_control_area(area),
        );
    }
    row
}
