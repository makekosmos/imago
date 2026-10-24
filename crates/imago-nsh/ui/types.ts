export type ThemeColors = {
  background: string;
  foreground: string;
  muted: string;
  border: string;
  accent: string;
  card: string;
};

export type Appearance = {
  settings: {
    mode: string;
    light_theme: string;
    dark_theme: string;
    accent_source: string;
    accent_color?: string;
    material: string;
    font_size: number;
  };
  policy: { following: boolean };
  capabilities: { materials: string[]; wallpaper_accent: boolean };
};

export type ThemeChoice = { key: string; name: string };
export type Shortcut = { name: string; label: string; binding: string };
export type IconName =
  | "about" | "check" | "inbox" | "logbook" | "more" | "nextWeek"
  | "plus" | "project" | "settings" | "someday" | "statistics" | "trash" | "week"
  | "sidebar" | "display";

export const spacing = {
  sidebarWidth: 240,
  titlebarHeight: 40,
  rowHeight: 32,
  compactRowHeight: 28,
  pageGutter: 28,
} as const;

export const typography = {
  body: 13,
  label: 12,
  title: 24,
  titlebar: 13,
} as const;
