import type { ApplicationTheme, ThemeColors } from "./types";

/**
 * Cobalt Foundry: forged steel greys lit by a cobalt arc. The Manager wears it
 * as its signature scheme; every consumer may ship its own palettes next to
 * this one, and the shape is the same so a choice can travel between apps.
 */
export const cobaltFoundry: ApplicationTheme = {
  background: "#090c15",
  shell: "#0d1019",
  card: "#101318",
  raised: "#181c25",
  input: "#1e222b",
  border: "#1e222b",
  foreground: "#ffffff",
  muted: "#717580",
  faint: "#8f9498",
  accent: "#1784fc",
  accentText: "#101318",
  danger: "#e67b71",
  dangerFill: "#3e1f24",
  success: "#3fbf7f",
};

/** The same cobalt palette for a light room: pale steel, same arc. */
export const cobaltFoundryLight: ApplicationTheme = {
  background: "#f7f8fa",
  shell: "#eff1f5",
  card: "#ffffff",
  raised: "#eef1f6",
  border: "#dde2ea",
  foreground: "#111820",
  muted: "#5f6a7a",
  faint: "#8c96a6",
  accent: "#1f6df5",
  accentText: "#ffffff",
  danger: "#d5373c",
  dangerFill: "#fdecec",
  success: "#1f8a5b",
};

/** Older surfaces take the six-token shape; the missing tokens fall back. */
export function themeColors(theme: ApplicationTheme): ThemeColors {
  return {
    background: theme.background,
    foreground: theme.foreground,
    muted: theme.muted,
    border: theme.border,
    accent: theme.accent,
    card: theme.card,
  };
}

/** A theme card paints a whole UI palette; these are the ones NSH ships. */
export type PaletteTheme = { key: string; name: string; colors: ApplicationTheme };

function palette(key: string, name: string, colors: ApplicationTheme): PaletteTheme {
  return { key, name, colors };
}

export const lightThemes: PaletteTheme[] = [
  palette("merino-light", "Merino Light", {
    ...cobaltFoundryLight,
    background: "#f5f3ef",
    shell: "#eeeae4",
    card: "#ffffff",
    raised: "#f1ede7",
    border: "#e0dad1",
    foreground: "#221f1b",
    muted: "#6b645b",
    faint: "#948c82",
    accent: "#2563eb",
    accentText: "#ffffff",
  }),
  palette("alpine-milk", "Alpine Milk", {
    ...cobaltFoundryLight,
    background: "#f7f9fb",
    shell: "#eef2f7",
    card: "#ffffff",
    raised: "#eaf0f7",
    border: "#dbe3ee",
    foreground: "#152030",
    muted: "#5b6a7d",
    faint: "#8794a8",
    accent: "#0f7bd4",
  }),
  palette("buttercream-diner", "Buttercream Diner", {
    ...cobaltFoundryLight,
    background: "#fdf8ea",
    shell: "#f8f0dc",
    card: "#fffdf4",
    raised: "#f7eed8",
    border: "#eadfc2",
    foreground: "#2a2113",
    muted: "#736450",
    faint: "#9c8f7c",
    accent: "#d1622a",
    danger: "#c2402f",
  }),
  palette("coastal-linen", "Coastal Linen", {
    ...cobaltFoundryLight,
    background: "#f3f8f7",
    shell: "#e9f2f1",
    card: "#ffffff",
    raised: "#e8f1f0",
    border: "#d5e4e2",
    foreground: "#132524",
    muted: "#566a69",
    faint: "#839796",
    accent: "#0f9aa8",
    success: "#149a72",
  }),
];

export const darkThemes: PaletteTheme[] = [
  palette("cinder-peak", "Cinder Peak", {
    ...cobaltFoundry,
    background: "#121010",
    shell: "#161313",
    card: "#1c1818",
    raised: "#241f1f",
    border: "#2e2727",
    foreground: "#f1eae7",
    muted: "#a2968f",
    faint: "#736762",
    accent: "#f0743f",
    danger: "#e5544b",
    dangerFill: "#2b1614",
  }),
  palette("cobalt-foundry", "Cobalt Foundry", cobaltFoundry),
  palette("ember-hollow", "Ember Hollow", {
    ...cobaltFoundry,
    background: "#0e0c09",
    shell: "#12100b",
    card: "#191509",
    raised: "#221c10",
    border: "#2d2515",
    foreground: "#f4ead6",
    muted: "#a6987c",
    faint: "#776b53",
    accent: "#f2a03d",
    danger: "#e2603f",
    dangerFill: "#2a1a10",
    success: "#9cbf4b",
  }),
  palette("heliotrope", "Heliotrope", {
    ...cobaltFoundry,
    background: "#0c0a12",
    shell: "#100d17",
    card: "#16121f",
    raised: "#1e1929",
    border: "#291f38",
    foreground: "#ece7f8",
    muted: "#9a90b4",
    faint: "#6d6488",
    accent: "#b07cff",
    danger: "#ef5d8b",
    dangerFill: "#291322",
  }),
];
