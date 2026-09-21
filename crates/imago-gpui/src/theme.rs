// Runtime color palette seeded from agenda-gpui's src/theme.rs (zeron
// themes, oklch→sRGB tables in palettes.rs). Sidebar bg is a step darker
// than the app bg. Adds apply()/colors() wiring so gpui-component widgets
// render the same palette instead of the library default.
#![allow(non_snake_case)]

use crate::palettes::THEMES;
use gpui::{rgb, App, Hsla, SharedString};
use gpui_component::theme::{Theme, ThemeConfig, ThemeConfigColors, ThemeMode};
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};

/// Resolved color set for one theme+mode (see src/palettes.rs).
pub struct Palette {
    pub bg: u32,
    pub fg: u32,
    pub muted_fg: u32,
    pub border: u32,
    pub accent: u32,
    pub accent_fg: u32,
    pub accent_dim: u32,
    pub secondary: u32,
    pub card: u32,
    pub popover: u32,
    pub sidebar_bg: u32,
    pub sidebar_divider: u32,
    pub destructive: u32,
    pub warn: u32,
    pub success: u32,
    pub qe_chip_bg: u32,
    pub qe_chip_fg: u32,
}

/// Selected theme index into palettes::THEMES.
static THEME_IDX: AtomicU8 = AtomicU8::new(0);
/// Resolved mode: 0 = light, 1 = dark. "System" is resolved at render time.
static MODE: AtomicU8 = AtomicU8::new(1);

pub fn set_theme(idx: usize) {
    THEME_IDX.store(idx.min(THEMES.len() - 1) as u8, Ordering::Relaxed);
}
pub fn set_mode(dark: bool) {
    MODE.store(dark as u8, Ordering::Relaxed);
}
pub fn is_dark() -> bool {
    MODE.load(Ordering::Relaxed) != 0
}

pub fn pal() -> &'static Palette {
    let def = &THEMES[THEME_IDX.load(Ordering::Relaxed) as usize];
    if is_dark() {
        &def.dark
    } else {
        &def.light
    }
}

pub fn BG() -> u32 {
    pal().bg
}
pub fn FG() -> u32 {
    pal().fg
}
pub fn MUTED_FG() -> u32 {
    pal().muted_fg
}
pub fn BORDER() -> u32 {
    pal().border
}
pub fn ACCENT() -> u32 {
    pal().accent
}
pub fn ACCENT_FG() -> u32 {
    pal().accent_fg
}
pub fn ACCENT_DIM() -> u32 {
    pal().accent_dim
}
pub fn SECONDARY() -> u32 {
    pal().secondary
}
pub fn CARD() -> u32 {
    pal().card
}
pub fn POPOVER() -> u32 {
    pal().popover
}
pub fn SIDEBAR_BG() -> u32 {
    pal().sidebar_bg
}
pub fn DESTRUCTIVE() -> u32 {
    pal().destructive
}
pub fn WARN() -> u32 {
    pal().warn
}
#[allow(dead_code)]
pub fn SUCCESS() -> u32 {
    pal().success
}
pub fn SIDEBAR_DIVIDER() -> u32 {
    pal().sidebar_divider
}
pub fn QE_CHIP_BG() -> u32 {
    pal().qe_chip_bg
}
pub fn QE_CHIP_FG() -> u32 {
    pal().qe_chip_fg
}
pub const TAG_GRAY: u32 = 0x7a7a7a;

pub fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Mix `top` over `bottom` with alpha a (color-mix(in srgb, top a%, bottom)).
pub fn mix(top: u32, a: f32, bottom: u32) -> Hsla {
    c(mix_u32(top, a, bottom))
}

fn mix_u32(top: u32, a: f32, bottom: u32) -> u32 {
    let tr = ((top >> 16) & 0xff) as f32;
    let tg = ((top >> 8) & 0xff) as f32;
    let tb = (top & 0xff) as f32;
    let br = ((bottom >> 16) & 0xff) as f32;
    let bg = ((bottom >> 8) & 0xff) as f32;
    let bb = (bottom & 0xff) as f32;
    let r = (tr * a + br * (1.0 - a)).round() as u32;
    let g = (tg * a + bg * (1.0 - a)).round() as u32;
    let b = (tb * a + bb * (1.0 - a)).round() as u32;
    (r << 16) | (g << 8) | b
}

/// Color with alpha.
pub fn rgba(hex: u32, a: f32) -> Hsla {
    let mut col: Hsla = rgb(hex).into();
    col.a = a;
    col
}

/// lerp two opaque colors, t ∈ [0,1].
pub fn lerp(a: u32, b: u32, t: f32) -> Hsla {
    let (r1, g1, b1) = ((a >> 16) & 0xff, (a >> 8) & 0xff, a & 0xff);
    let (r2, g2, b2) = ((b >> 16) & 0xff, (b >> 8) & 0xff, b & 0xff);
    let r = (r1 as f32 + (r2 as f32 - r1 as f32) * t).round() as u32;
    let g = (g1 as f32 + (g2 as f32 - g1 as f32) * t).round() as u32;
    let bb = (b1 as f32 + (b2 as f32 - b1 as f32) * t).round() as u32;
    c((r << 16) | (g << 8) | bb)
}

pub fn tag_color(name: &str) -> u32 {
    match name {
        "red" => 0xe66f64,
        "orange" => 0xd9975c,
        "yellow" => 0xccae59,
        "green" => 0x3db07c,
        "blue" => 0x548bd0,
        "purple" => 0xa074d0,
        "pink" => 0xd981ac,
        "gray" | "grey" => TAG_GRAY,
        _ => MUTED_FG(),
    }
}

/// foreground over background mixes (light theme: over white).
pub fn fg_mix(a: f32) -> Hsla {
    mix(FG(), a, BG())
}
pub fn border_mix(a: f32) -> Hsla {
    mix(BORDER(), a, BG())
}
pub fn muted_fg_mix(a: f32) -> Hsla {
    mix(MUTED_FG(), a, BG())
}

/// cubic-bezier(x1,y1,x2,y2) easing via bisection on x.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let sample = |t: f32| -> (f32, f32) {
        let u = 1.0 - t;
        (
            3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t,
            3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t,
        )
    };
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    let mut t = x;
    for _ in 0..32 {
        let (sx, _) = sample(t);
        if (sx - x).abs() < 1e-5 {
            break;
        }
        if sx < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) / 2.0;
    }
    sample(t).1
}

/// Sidebar & emphasized easing: cubic-bezier(0.22, 1, 0.36, 1).
pub fn ease_emphasized(x: f32) -> f32 {
    cubic_bezier(0.22, 1.0, 0.36, 1.0, x)
}

/// Standard ease for 120ms hover transitions (CSS `ease`).
pub fn ease_standard(x: f32) -> f32 {
    cubic_bezier(0.25, 0.1, 0.25, 1.0, x)
}

/// Sliding hover-highlight move: cubic-bezier(0.23, 1, 0.32, 1).
pub fn ease_out_quint(x: f32) -> f32 {
    cubic_bezier(0.23, 1.0, 0.32, 1.0, x)
}

// ---------------------------------------------------------------------------
// gpui-component theme wiring
// ---------------------------------------------------------------------------

/// Install the selected theme's light+dark palettes as the app's
/// gpui-component themes and apply the active mode. Call once after
/// `gpui_component::init(cx)`, after any `set_theme`/`set_mode` calls.
pub fn apply(cx: &mut App) {
    let def = &THEMES[THEME_IDX.load(Ordering::Relaxed) as usize];
    let theme = Theme::global_mut(cx);
    theme.light_theme = Rc::new(config(&def.light, ThemeMode::Light));
    theme.dark_theme = Rc::new(config(&def.dark, ThemeMode::Dark));
    let mode = if is_dark() {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    Theme::change(mode, None, cx);
}

fn config(p: &Palette, mode: ThemeMode) -> ThemeConfig {
    ThemeConfig {
        name: if mode.is_dark() {
            "Imago Dark".into()
        } else {
            "Imago Light".into()
        },
        mode,
        // fs13 like the Vue/Agenda shells; controls get 8px/10px radii.
        font_size: Some(13.0),
        font_family: None,
        mono_font_family: None,
        mono_font_size: None,
        radius: Some(8),
        radius_lg: Some(10),
        shadow: Some(true),
        colors: colors(p),
        ..Default::default()
    }
}

fn hex(v: u32) -> Option<SharedString> {
    Some(format!("#{v:06x}").into())
}

fn hexa(v: u32, a: f32) -> Option<SharedString> {
    let byte = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
    Some(format!("#{v:06x}{byte:02x}").into())
}

fn mix_hex(fg: u32, w: f32, bg: u32) -> Option<SharedString> {
    hex(mix_u32(fg, w, bg))
}

/// Palette → gpui-component `ThemeConfigColors`.
///
/// Hover/active shades derive from the palette: accent controls fade toward
/// `accent_fg` (lightens dark accents, darkens light ones); neutral fills
/// are `fg` alpha steps — the same `rgba(FG, …)` recipe agenda chrome uses.
fn colors(p: &Palette) -> ThemeConfigColors {
    let fg = p.fg;
    let bg = p.bg;
    let white = 0xffffff;
    let black = 0x000000;

    let mut cc = ThemeConfigColors::default();
    cc.accent = hex(p.accent);
    cc.accent_foreground = hex(p.accent_fg);
    cc.accordion = hex(p.card);
    cc.background = hex(bg);
    cc.border = hex(p.border);

    // Default button: neutral foreground-tinted fill (agenda rgba(FG, ·)).
    cc.button = hexa(fg, 0.06);
    cc.button_hover = hexa(fg, 0.10);
    cc.button_active = hexa(fg, 0.16);
    cc.button_foreground = hex(fg);

    // Primary = accent fill.
    cc.button_primary = hex(p.accent);
    cc.button_primary_foreground = hex(p.accent_fg);
    cc.button_primary_hover = mix_hex(p.accent, 0.85, p.accent_fg);
    cc.button_primary_active = mix_hex(p.accent, 0.72, p.accent_fg);

    // Secondary = `secondary` surface with fg-tinted hover.
    cc.button_secondary = hex(p.secondary);
    cc.button_secondary_foreground = hex(fg);
    cc.button_secondary_hover = mix_hex(fg, 0.10, p.secondary);
    cc.button_secondary_active = mix_hex(fg, 0.16, p.secondary);

    // Danger/success/info/warning buttons: tinted-outline styles.
    cc.button_danger = hexa(p.destructive, 0.10);
    cc.button_danger_foreground = hex(p.destructive);
    cc.button_danger_hover = hexa(p.destructive, 0.20);
    cc.button_danger_active = hexa(p.destructive, 0.35);
    cc.button_success = hexa(p.success, 0.20);
    cc.button_success_foreground = hex(p.success);
    cc.button_success_hover = hexa(p.success, 0.28);
    cc.button_success_active = hexa(p.success, 0.38);
    cc.button_info = hexa(p.accent, 0.15);
    cc.button_info_foreground = hex(p.accent);
    cc.button_info_hover = hexa(p.accent, 0.25);
    cc.button_info_active = hexa(p.accent, 0.35);
    cc.button_warning = hexa(p.warn, 0.15);
    cc.button_warning_foreground = hex(p.warn);
    cc.button_warning_hover = hexa(p.warn, 0.25);
    cc.button_warning_active = hexa(p.warn, 0.35);

    cc.caret = hex(fg);
    cc.chart_bullish = hex(p.success);
    cc.chart_bearish = hex(p.destructive);

    cc.danger = hex(p.destructive);
    cc.danger_foreground = hex(white);
    cc.danger_hover = mix_hex(p.destructive, 0.88, white);
    cc.danger_active = mix_hex(p.destructive, 0.80, black);
    cc.info = hex(p.accent);
    cc.info_foreground = hex(p.accent_fg);
    cc.info_hover = mix_hex(p.accent, 0.88, white);
    cc.info_active = mix_hex(p.accent, 0.80, black);
    cc.success = hex(p.success);
    cc.success_foreground = hex(white);
    cc.success_hover = mix_hex(p.success, 0.88, white);
    cc.success_active = mix_hex(p.success, 0.80, black);
    cc.warning = hex(p.warn);
    cc.warning_foreground = hex(black);
    cc.warning_hover = mix_hex(p.warn, 0.88, white);
    cc.warning_active = mix_hex(p.warn, 0.80, black);

    cc.description_list_label = hex(p.card);
    cc.description_list_label_foreground = hex(p.muted_fg);
    cc.drag_border = hex(p.accent);
    cc.drop_target = hexa(p.accent, 0.20);
    cc.foreground = hex(fg);

    cc.group_box = hex(p.card);
    cc.group_box_foreground = hex(fg);
    cc.group_box_title_foreground = hex(p.muted_fg);

    cc.input = hex(p.border);
    cc.link = hex(p.accent);
    cc.link_hover = mix_hex(p.accent, 0.85, p.accent_fg);
    cc.link_active = mix_hex(p.accent, 0.72, p.accent_fg);

    cc.list = hex(bg);
    cc.list_hover = hexa(fg, 0.05);
    cc.list_active = hexa(fg, 0.12);
    cc.list_active_border = hex(p.border);
    cc.list_even = hex(bg);
    cc.list_head = mix_hex(fg, 0.06, bg);

    cc.muted = hex(p.card);
    cc.muted_foreground = hex(p.muted_fg);
    cc.popover = hex(p.popover);
    cc.popover_foreground = hex(fg);
    cc.primary = hex(p.accent);
    cc.primary_foreground = hex(p.accent_fg);
    cc.primary_hover = mix_hex(p.accent, 0.85, p.accent_fg);
    cc.primary_active = mix_hex(p.accent, 0.72, p.accent_fg);
    cc.progress_bar = hex(p.accent);
    cc.ring = hex(p.accent);

    cc.scrollbar = hexa(0, 0.0);
    cc.scrollbar_thumb = hexa(fg, 0.18);
    cc.scrollbar_thumb_hover = hexa(fg, 0.32);

    cc.secondary = hex(p.secondary);
    cc.secondary_foreground = hex(fg);
    cc.secondary_hover = mix_hex(fg, 0.10, p.secondary);
    cc.secondary_active = mix_hex(fg, 0.16, p.secondary);
    cc.selection = hexa(p.accent, 0.30);

    cc.sidebar = hex(p.sidebar_bg);
    cc.sidebar_foreground = hex(fg);
    cc.sidebar_border = hex(p.sidebar_divider);
    cc.sidebar_primary = hex(p.accent);
    cc.sidebar_primary_foreground = hex(p.accent_fg);
    cc.sidebar_accent = hexa(fg, 0.12);
    cc.sidebar_accent_foreground = hex(fg);

    cc.skeleton = hexa(fg, 0.10);
    cc.slider_bar = hex(p.secondary);
    cc.slider_thumb = hex(fg);
    cc.switch = hexa(fg, 0.18);
    cc.switch_thumb = hex(fg);

    cc.tab = hexa(0, 0.0);
    cc.tab_foreground = hex(p.muted_fg);
    cc.tab_active = hex(p.card);
    cc.tab_active_foreground = hex(fg);
    cc.tab_bar = hex(bg);
    cc.tab_bar_segmented = hex(p.secondary);

    cc.table = hex(p.card);
    cc.table_row_border = hex(p.sidebar_divider);
    cc.table_hover = hexa(fg, 0.05);
    cc.table_active = hexa(fg, 0.12);
    cc.table_active_border = hex(p.border);
    cc.table_even = hex(p.card);
    cc.table_head = mix_hex(fg, 0.06, p.card);
    cc.table_foot = hex(p.card);
    cc.table_foot_foreground = hex(p.muted_fg);

    cc.title_bar = hex(bg);
    cc.title_bar_border = hex(p.border);
    cc.status_bar = hex(p.sidebar_bg);
    cc.status_bar_border = hex(p.sidebar_divider);

    cc.overlay = hexa(black, 0.45);
    cc.window_border = hex(p.sidebar_divider);
    cc
}
