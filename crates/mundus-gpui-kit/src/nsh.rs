use gpui::{Context, Window};
use nsh_platform_gpui::{App as NshApp, AppOptions, AppView};

/// Creates a GPUI host for an NSH bundle and configures native components
/// before the bundle starts running.
pub fn mount_app_view(
    window: &mut Window,
    cx: &mut Context<AppView>,
    app: NshApp,
    configure: impl FnOnce(&mut AppView),
) -> AppView {
    let mut view = AppView::new(AppOptions::default(), window, cx);
    configure(&mut view);
    view.load(app);
    view
}
