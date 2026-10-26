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
    font_family: string;
  };
  policy: { following: boolean };
  capabilities: { materials: string[]; wallpaper_accent: boolean };
};

export type ThemeChoice = { key: string; name: string };
export type Shortcut = { name: string; label: string; binding: string };
export type TitlebarAction = { id: string; label: string; icon: IconName };
export type IconName =
  | "about" | "check" | "inbox" | "logbook" | "more" | "nextWeek"
  | "plus" | "project" | "settings" | "someday" | "statistics" | "trash" | "week"
  | "sidebar" | "display" | "windowMin" | "windowMax" | "windowClose";

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

export const defaultColors: ThemeColors = {
  background: "#0d0c10",
  foreground: "#e7e5ea",
  muted: "#8d8a94",
  border: "#27262a",
  accent: "#af94f9",
  card: "#131216",
};

export const accentChoices = ["#8ec07c", "#d79921", "#b18aff", "#83a598", "#fb4934", "#d3869b", "#458588", "#fabd2f"] as const;
export const densityChoices = [[12, "Компактная"], [13, "Обычная"], [15, "Свободная"]] as const;
