//! Embedded Agenda Inter font and Hugeicons (core-free-icons 4.3.2).
use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

pub struct Assets;
const ICONS: &[(&str, &[u8])] = &[
    (
        "icons/database.svg",
        include_bytes!("../assets/icons/database.svg"),
    ),
    ("icons/sync.svg", include_bytes!("../assets/icons/sync.svg")),
    (
        "icons/store.svg",
        include_bytes!("../assets/icons/store.svg"),
    ),
    ("icons/cpu.svg", include_bytes!("../assets/icons/cpu.svg")),
    (
        "icons/connections.svg",
        include_bytes!("../assets/icons/connections.svg"),
    ),
    (
        "icons/download.svg",
        include_bytes!("../assets/icons/download.svg"),
    ),
    ("icons/key.svg", include_bytes!("../assets/icons/key.svg")),
    (
        "icons/globe.svg",
        include_bytes!("../assets/icons/globe.svg"),
    ),
    ("icons/code.svg", include_bytes!("../assets/icons/code.svg")),
    (
        "icons/settings.svg",
        include_bytes!("../assets/icons/settings.svg"),
    ),
    (
        "icons/sidebar-left.svg",
        include_bytes!("../assets/icons/sidebar-left.svg"),
    ),
    (
        "icons/help-circle.svg",
        include_bytes!("../assets/icons/help-circle.svg"),
    ),
    (
        "icons/window-min.svg",
        include_bytes!("../assets/icons/window-min.svg"),
    ),
    (
        "icons/window-max.svg",
        include_bytes!("../assets/icons/window-max.svg"),
    ),
    (
        "icons/window-restore.svg",
        include_bytes!("../assets/icons/window-restore.svg"),
    ),
    (
        "icons/status-x.svg",
        include_bytes!("../assets/icons/status-x.svg"),
    ),
];

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = ICONS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(*bytes)));
        }
        gpui::assets::Assets.load(path)
    }
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui::assets::Assets.list(path)?;
        paths.extend(
            ICONS
                .iter()
                .filter(|(name, _)| name.starts_with(path))
                .map(|(name, _)| SharedString::new_static(name)),
        );
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}

pub fn font_bytes() -> Vec<Cow<'static, [u8]>> {
    vec![Cow::Borrowed(include_bytes!("../assets/fonts/Inter.ttf"))]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_icons_resolve_and_unknown_assets_fail() {
        let paths = Assets.list("icons/").unwrap();
        assert!(paths.len() >= ICONS.len());
        for path in paths {
            let bytes = Assets.load(&path).unwrap().unwrap();
            assert!(std::str::from_utf8(&bytes).unwrap().contains("<svg"));
        }
        assert!(Assets.load("missing.svg").is_err());
        assert!(Assets.list("missing/").unwrap().is_empty());
        assert!(!font_bytes()[0].is_empty());
    }
}
