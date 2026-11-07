export type ThemeColors = {
  background: string;
  foreground: string;
  muted: string;
  border: string;
  accent: string;
  card: string;
};

export type IconName =
  | "about" | "check" | "inbox" | "logbook" | "more" | "nextWeek"
  | "plus" | "project" | "settings" | "someday" | "statistics" | "trash" | "week"
  | "sidebar" | "display" | "windowMin" | "windowMax" | "windowClose"
  | "server" | "info" | "palette" | "apps" | "plug" | "database" | "pulse"
  | "monitor" | "globe" | "key" | "chip" | "gear" | "code" | "search"
  | "power" | "chevron" | "refresh" | "shield" | "external" | "mundus";

/**
 * Settings surfaces read more tokens than the six of `ThemeColors`: a window
 * shell distinct from the page, a raised fill for inputs, a third ink for
 * captions and the status colors a pill or a dot needs.
 */
export type ApplicationTheme = ThemeColors & {
  /** Field and chip fill, a step above the card. */
  input?: string;
  shell: string;
  raised: string;
  faint: string;
  accentText: string;
  danger: string;
  dangerFill: string;
  success: string;
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
export type SidebarEntry = { route: string; label: string; icon: IconName };
/** A sidebar row in the settings layout: an accent tile, a label, a caption. */
export type NavEntry = SidebarEntry & { accent: string; accentTo?: string; caption?: string };
export type NavGroup = { id: string; entries: NavEntry[] };
export type ProjectEntry = { id: string; title: string };
export type TitlebarAction = { id: string; label: string; icon: IconName };

export const spacing = {
  sidebarWidth: 240,
  titlebarHeight: 40,
  rowHeight: 32,
  compactRowHeight: 28,
  pageGutter: 28,
} as const;

/** Metrics of the settings shell the Manager and NSH preference panes wear. */
export const settingsMetrics = {
  sidebarWidth: 282,
  sidebarPadding: 8,
  sidebarTop: 16,
  searchHeight: 32,
  searchGap: 8,
  navRowHeight: 40,
  navGroupGap: 14,
  navEntryGap: 2,
  tileSize: 24,
  tileRadius: 6,
  logoRowHeight: 50,
  logoRowPadding: 7,
  logoTileSize: 36,
  cardRadius: 10,
  cardRowHeight: 51,
  cardDenseRowHeight: 50,
  cardPaddingX: 10,
  cardPaddingY: 8,
  cardDensePaddingY: 6,
  fieldRowHeight: 48,
  fieldLabelWidth: 62,
  pageGutter: 16,
  pageMaxWidth: 1180,
  pageTop: 12,
  pageBottom: 56,
  headerHeight: 26,
  headerGap: 18,
  sectionGap: 40,
  labelGap: 8,
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
