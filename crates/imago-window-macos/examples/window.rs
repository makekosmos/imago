//! Opens the sidebar window with an empty nsh surface as its content.

#![cfg(target_os = "macos")]

use imago_native_sidebar::Row;
use nsh_core::ui::prelude::*;
use nsh_platform_macos::{AppView, Root};

#[component]
fn Empty() -> impl IntoElement {
    view().w_full().h_full().child(text("nsh content"))
}

fn register_components(_view: &AppView) {}

fn main() -> std::process::ExitCode {
    imago_window_macos::run(
        "Prototype",
        (900.0, 600.0),
        vec![
            Row {
                id: "home",
                label: "Главная".into(),
                symbol: Some("house"),
            },
            Row {
                id: "all",
                label: "Все книги".into(),
                symbol: Some("books.vertical"),
            },
        ],
        Root::of::<Empty>,
        Root::of::<Empty>,
        register_components,
    )
}
