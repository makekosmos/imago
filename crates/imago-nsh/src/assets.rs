use std::borrow::Cow;

use gpui::Result;

const ICONS: &[(&str, &[u8])] = &[
    (
        "icons/analytics-01.svg",
        include_bytes!("../ui/assets/icons/analytics-01.svg"),
    ),
    (
        "icons/book-open.svg",
        include_bytes!("../ui/assets/icons/book-open.svg"),
    ),
    (
        "icons/calendar-01.svg",
        include_bytes!("../ui/assets/icons/calendar-01.svg"),
    ),
    (
        "icons/calendar-02.svg",
        include_bytes!("../ui/assets/icons/calendar-02.svg"),
    ),
    (
        "icons/check.svg",
        include_bytes!("../ui/assets/icons/check.svg"),
    ),
    (
        "icons/clock-01.svg",
        include_bytes!("../ui/assets/icons/clock-01.svg"),
    ),
    (
        "icons/delete.svg",
        include_bytes!("../ui/assets/icons/delete.svg"),
    ),
    (
        "icons/folder.svg",
        include_bytes!("../ui/assets/icons/folder.svg"),
    ),
    (
        "icons/help-circle.svg",
        include_bytes!("../ui/assets/icons/help-circle.svg"),
    ),
    (
        "icons/inbox.svg",
        include_bytes!("../ui/assets/icons/inbox.svg"),
    ),
    (
        "icons/more-circle.svg",
        include_bytes!("../ui/assets/icons/more-circle.svg"),
    ),
    (
        "icons/plus.svg",
        include_bytes!("../ui/assets/icons/plus.svg"),
    ),
    (
        "icons/preference-vertical.svg",
        include_bytes!("../ui/assets/icons/preference-vertical.svg"),
    ),
    (
        "icons/sidebar-left.svg",
        include_bytes!("../ui/assets/icons/sidebar-left.svg"),
    ),
    (
        "icons/sliders.svg",
        include_bytes!("../ui/assets/icons/sliders.svg"),
    ),
];

pub(crate) fn load(path: &str) -> Result<Option<Cow<'static, [u8]>>> {
    let path = path.replace('\\', "/");
    Ok(ICONS
        .iter()
        .find(|(name, _)| path.ends_with(name))
        .map(|(_, bytes)| Cow::Borrowed(*bytes)))
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn resolves_package_svg_asset_urls() {
        let svg = load("C:/workspace/imago/crates/imago-nsh/ui/assets/icons/inbox.svg")
            .unwrap()
            .unwrap();
        assert!(std::str::from_utf8(&svg).unwrap().contains("<svg"));
    }
}
