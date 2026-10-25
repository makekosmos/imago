//! Opens the sidebar window with an empty nsh surface as its content.

#[cfg(target_os = "macos")]
use imago_native_sidebar::Row;
#[cfg(target_os = "macos")]
use nsh_core::ui::prelude::*;
#[cfg(target_os = "macos")]
use nsh_platform_macos::Root;

#[cfg(target_os = "macos")]
#[component]
fn Empty() -> impl IntoElement {
    view().w_full().h_full().child(text("nsh content"))
}

#[cfg(target_os = "macos")]
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
        |_| {},
    )
}

#[cfg(not(target_os = "macos"))]
fn main() {}
