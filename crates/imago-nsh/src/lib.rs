use gpui::{Context, Window};
use nsh_platform_gpui::AppOptions;
mod assets;

pub use imago_gpui;
pub use mundus_gpui_kit;
pub use nsh_base::{registry as nsh_base_registry, Theme as NshTheme};
pub use nsh_platform_gpui::{App as NshApp, AppView, Bundle, ComponentContext, NativeComponent};

pub fn load_ui_asset(path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
    assets::load(path)
}

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
