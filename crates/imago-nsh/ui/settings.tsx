import { Scroll, Text, View } from "nsh";
import { Input } from "nsh/base";
import { titleBar } from "nsh/desktop";
import { Icon } from "./components";
import type { ApplicationTheme, IconName, NavGroup } from "./types";
import { settingsMetrics as M } from "./types";

/** One hex colour nudged toward another: the sheen and the shade of a tile. */
function mix(hex: string, toward: string, amount: number): string {
  const parse = (value: string) => {
    const clean = value.replace("#", "");
    return [0, 2, 4].map((index) => parseInt(clean.slice(index, index + 2), 16));
  };
  const from = parse(hex);
  const to = parse(toward);
  const channel = (index: number) => {
    const value = Math.round(from[index] + (to[index] - from[index]) * amount);
    return Math.max(0, Math.min(255, value)).toString(16).padStart(2, "0");
  };
  return `#${[0, 1, 2].map(channel).join("")}`;
}

/** Piecewise interpolation across a colour list. */
function along(colours: string[], t: number): string {
  const span = colours.length - 1;
  const at = Math.min(span - 1, Math.floor(t * span));
  const local = t * span - at;
  return mix(colours[at], colours[at + 1], local);
}

/**
 * A tile's fill. The design paints a white sheen over an accent pair and shades
 * the foot; NSH paints one background image, so the same three layers are
 * sampled into stops here — thirteen of them, which is what a 24pt tile needs
 * for the gloss to read as a gloss.
 */
function tileGradient(accent: string, accentTo?: string): string {
  // "Внешний вид" wears the whole spectrum of the accent family.
  const colours = accentTo === "spectrum"
    ? ["#f79411", "#13c370", "#1784fc", "#915bfc", "#fb3499"]
    : [accent, accentTo ?? accent];
  const stops = Array.from({ length: 13 }, (_, index) => {
    const t = index / 12;
    const base = along(colours, t);
    // White 23% at the top fading out by the middle, black 17% at the foot.
    const sheen = t <= 0.5 ? 0.23 * (1 - t / 0.5) : -0.17 * ((t - 0.5) / 0.5);
    const colour = sheen >= 0 ? mix(base, "#ffffff", sheen) : mix(base, "#000000", -sheen);
    return `${colour} ${Math.round(t * 100)}%`;
  });
  return `linear-gradient(180deg, ${stops.join(", ")})`;
}

/** A 24pt gradient tile with a white glyph — the sidebar's colour language. */
export function IconTile(props: { icon: IconName; accent: string; accentTo?: string; size?: number; radius?: number }) {
  const size = props.size ?? M.tileSize;
  return <View class="items-center justify-center" style={{
    width: size,
    height: size,
    borderRadius: props.radius ?? M.tileRadius,
    backgroundImage: tileGradient(props.accent, props.accentTo),
    borderWidth: 1,
    borderColor: "rgba(255,255,255,0.20)",
    boxShadow: "0 1px 2px rgba(0,0,0,0.32)",
  }}>
    <Icon name={props.icon} size={Math.round(size * 0.72)} color="#ffffff" />
  </View>;
}

/** The Mundus mark in a pill of its own — the running server's avatar. */
export function LogoTile(props: { size?: number }) {
  const size = props.size ?? M.logoTileSize;
  return <View class="items-center justify-center" style={{
    width: size,
    height: size,
    borderRadius: size / 2,
    backgroundImage: "linear-gradient(135deg, #93aeea 0%, #5b83d6 34%, #359ecb 100%)",
    borderWidth: 1,
    borderColor: "rgba(255,255,255,0.22)",
  }}>
    <Icon name="mundus" size={Math.round(size * 0.56)} color="#ffffff" />
  </View>;
}

/** Sidebar row: accent tile, label and an optional caption under it. */
export function NavTile(props: {
  label: string;
  icon: IconName;
  accent: string;
  accentTo?: string;
  caption?: string;
  logo?: boolean;
  selected: boolean;
  colors: ApplicationTheme;
  onPress: () => void;
}) {
  return <View onPress={props.onPress} role="button" accessibilityLabel={props.label} accessibilityState={{ selected: props.selected }}
    class="flex-row items-center gap-2.5 rounded-[10px]"
    style={{
      height: props.logo ? M.logoRowHeight : M.navRowHeight,
      paddingLeft: M.cardPaddingX,
      paddingRight: M.cardPaddingX,
      backgroundColor: props.selected ? props.colors.raised : "transparent",
    }}>
    {/* The design's tile sits half a point above the row's centre: at 24 of 40
        the rounding lands on the same pixel as its label only with this nudge. */}
    <View style={{ marginTop: props.logo ? 0 : -1 }}>
      {props.logo
        ? <LogoTile />
        : <IconTile icon={props.icon} accent={props.accent} accentTo={props.accentTo} />}
    </View>
    <View class="flex-1" style={{ gap: 4 }}>
      <Text style={{ fontSize: 14, fontWeight: "500", lineHeight: 20, color: props.colors.foreground }}>{props.label}</Text>
      {props.caption
        ? <Text style={{ fontSize: 12, lineHeight: 16, fontWeight: "500", color: props.colors.muted }}>{props.caption}</Text>
        : null}
    </View>
  </View>;
}

export function SearchField(props: { value: string; placeholder: string; colors: ApplicationTheme; onChange: (value: string) => void }) {
  return <View class="w-full flex-row items-center gap-2 rounded-[10px] px-2.5" style={{ height: M.searchHeight, backgroundColor: props.colors.input ?? props.colors.raised }}>
    <Icon name="search" size={16} color={props.colors.faint} />
    <Input value={props.value} onChange={(event: { value: string }) => props.onChange(event.value)} placeholder={props.placeholder} accessibilityLabel={props.placeholder}
      class="flex-1 text-[13px]" style={{ backgroundColor: "transparent", color: props.colors.foreground }} />
  </View>;
}

export function SettingsSidebar(props: {
  colors: ApplicationTheme;
  groups: NavGroup[];
  current: string;
  search: string;
  onSearch: (value: string) => void;
  onNavigate: (route: string) => void;
}) {
  const matches = (entry: { label: string }) => !props.search.trim() || entry.label.toLowerCase().includes(props.search.trim().toLowerCase());
  const lights = titleBar();
  const top = lights.left > 0 ? lights.height + 4 : M.sidebarTop;
  return <View class="h-full shrink-0 flex-col" style={{ width: M.sidebarWidth, backgroundColor: props.colors.shell, paddingTop: top, paddingBottom: M.sidebarTop, paddingLeft: M.sidebarPadding, paddingRight: M.sidebarPadding }}>
    <View class="min-h-0 flex-1 flex-col" style={{ gap: M.searchGap }}>
      <SearchField value={props.search} placeholder="Поиск настроек" colors={props.colors} onChange={props.onSearch} />
      <Scroll class="min-h-0 flex-1">
        <View style={{ gap: M.navGroupGap }}>
          {props.groups.map((group, index) => {
            const entries = group.entries.filter(matches);
            if (!entries.length) return null;
            // The design leaves thirteen points above the first group, not fourteen.
            return <View key={group.id} style={{ gap: M.navEntryGap, marginBottom: index === 0 ? -1 : 0 }}>
              {entries.map((entry) => <NavTile key={entry.route} label={entry.label} icon={entry.icon}
                accent={entry.accent} accentTo={entry.accentTo} caption={entry.caption} logo={entry.route === "server"}
                selected={props.current === entry.route} colors={props.colors} onPress={() => props.onNavigate(entry.route)} />)}
            </View>;
          })}
        </View>
      </Scroll>
    </View>
  </View>;
}

/**
 * Page frame: the scroll area, its gutter and the column the cards sit in. The
 * page's own heading is the window's top row — dragging it moves the window, and
 * the window's controls hang off it when the platform does not paint its own.
 */
export function SettingsPage(props: { colors: ApplicationTheme; title?: string; actions?: any; children: any }) {
  const controls = titleBar();
  return <Scroll class="min-h-0 flex-1" style={{ backgroundColor: props.colors.background }}>
    <View class="w-full" style={{ maxWidth: M.pageMaxWidth, paddingLeft: M.pageGutter, paddingRight: M.pageGutter, paddingTop: M.pageTop, paddingBottom: M.pageBottom }}>
      <View class="flex-row items-center" style={{ height: M.headerHeight, paddingLeft: controls.left }} desktop={{ windowDrag: true }}>
        {props.title ? <Text style={{ fontSize: 13, lineHeight: 19, fontWeight: "500", color: props.colors.foreground }}>{props.title}</Text> : null}
        {props.actions ?? null}
        <View class="flex-1" />
      </View>
      <View style={{ gap: M.sectionGap, marginTop: M.headerGap }}>{props.children}</View>
    </View>
  </Scroll>;
}

export function SectionLabel(props: { children: string; colors: ApplicationTheme; right?: any }) {
  return <View class="flex-row items-center justify-between" style={{ marginBottom: M.labelGap, paddingLeft: M.cardPaddingX }}>
    <Text style={{ fontSize: 13, lineHeight: 18, fontWeight: "500", color: props.colors.foreground }}>{props.children}</Text>
    {props.right ?? null}
  </View>;
}

export function Card(props: { colors: ApplicationTheme; children: any; padded?: boolean }) {
  return <View class="overflow-hidden" style={{ backgroundColor: props.colors.card, borderRadius: M.cardRadius, padding: props.padded ? 12 : 0 }}>
    {props.children}
  </View>;
}

/** One card row: optional avatar, title over an optional subtitle. */
export function CardRow(props: {
  colors: ApplicationTheme;
  title: string;
  subtitle?: string;
  icon?: IconName;
  accent?: string;
  accentTo?: string;
  /** Flat avatar fill, when a gradient pair is not what the row wants. */
  iconFill?: string;
  logo?: boolean;
  right?: any;
  onPress?: () => void;
  last?: boolean;
}) {
  return <View onPress={props.onPress} role={props.onPress ? "button" : undefined}
    class="flex-row items-center gap-2.5"
    style={{
      height: props.logo || props.icon ? M.cardRowHeight : undefined,
      minHeight: M.cardDenseRowHeight,
      paddingLeft: M.cardPaddingX,
      paddingRight: M.cardPaddingX,
      paddingTop: props.logo || props.icon ? 0 : M.cardDensePaddingY,
      paddingBottom: props.logo || props.icon ? 0 : M.cardDensePaddingY,
      borderBottomWidth: props.last ? 0 : 1,
      borderColor: props.colors.border,
    }}>
    {props.logo
      ? <LogoTile />
      : props.icon
        ? <View class="items-center justify-center" style={{
            width: 35,
            height: 35,
            borderRadius: 18,
            backgroundColor: props.iconFill,
            backgroundImage: props.iconFill
              ? undefined
              : props.accent
                ? tileGradient(props.accent, props.accentTo)
                : `linear-gradient(-135deg, #151515 0%, #303030 100%)`,
            borderWidth: 1,
            borderColor: "#00000033",
          }}>
            <Icon name={props.icon} size={20} color={props.accent ? "#ffffff" : "#bfbfbf"} />
          </View>
        : null}
    <View class="flex-1" style={{ gap: 4 }}>
      <Text style={{ fontSize: 13, lineHeight: 18, fontWeight: "600", color: props.colors.foreground }}>{props.title}</Text>
      {props.subtitle ? <Text style={{ fontSize: 12, lineHeight: 16, fontWeight: "500", color: props.colors.muted }}>{props.subtitle}</Text> : null}
    </View>
    {props.right ?? null}
  </View>;
}

/** Label on the left, control on the right — the "Название" row of settings. */
export function FieldRow(props: { label: string; colors: ApplicationTheme; labelWidth?: number; children: any; last?: boolean }) {
  return <View class="flex-row items-center" style={{
    height: M.fieldRowHeight,
    gap: 40,
    paddingLeft: M.cardPaddingX,
    paddingRight: M.cardPaddingX,
    paddingTop: M.cardPaddingY,
    paddingBottom: M.cardPaddingY,
    borderBottomWidth: props.last ? 0 : 1,
    borderColor: props.colors.border,
  }}>
    <Text style={{ width: props.labelWidth ?? M.fieldLabelWidth, fontSize: 13, fontWeight: "500", color: props.colors.foreground }}>{props.label}</Text>
    <View class="flex-1">{props.children}</View>
  </View>;
}

export function TextInput(props: { value: string; placeholder: string; colors: ApplicationTheme; onChange: (value: string) => void; onSubmit?: (value: string) => void; accessibilityLabel?: string }) {
  return <Input value={props.value} onChange={(event: { value: string }) => props.onChange(event.value)} onSubmit={props.onSubmit ? (event: { text: string }) => props.onSubmit?.(event.text) : undefined}
    placeholder={props.placeholder} accessibilityLabel={props.accessibilityLabel ?? props.placeholder}
    class="h-8 rounded-[10px] px-2.5 text-[13px]" style={{ backgroundColor: props.colors.input ?? props.colors.raised, color: props.colors.foreground }} />;
}

export function PillButton(props: { label: string; colors: ApplicationTheme; tone?: "accent" | "danger" | "neutral"; onPress: () => void; disabled?: boolean }) {
  const tone = props.tone ?? "neutral";
  const background = tone === "accent" ? props.colors.accent : tone === "danger" ? props.colors.dangerFill : props.colors.raised;
  const color = tone === "accent" ? props.colors.accentText : tone === "danger" ? props.colors.danger : props.colors.foreground;
  return <View onPress={props.disabled ? undefined : props.onPress} role="button" accessibilityLabel={props.label}
    accessibilityState={{ disabled: !!props.disabled }} hoverStyle={{ opacity: 0.9 }}
    class="items-center justify-center rounded-full" style={{ paddingLeft: 8, paddingRight: 8, paddingTop: 4, paddingBottom: 4, backgroundColor: background, opacity: props.disabled ? 0.5 : 1 }}>
    <Text style={{ fontSize: 13, lineHeight: 15, fontWeight: "500", color }}>{props.label}</Text>
  </View>;
}

export function Switch(props: { value: boolean; colors: ApplicationTheme; onChange: (value: boolean) => void; accessibilityLabel?: string }) {
  const off = !props.value;
  return <View onPress={() => props.onChange(!props.value)} role="switch" accessibilityLabel={props.accessibilityLabel}
    accessibilityState={{ checked: props.value }} class="justify-center"
    style={{
      width: 44,
      height: 22,
      marginTop: 1,
      borderRadius: 11,
      backgroundColor: props.value ? props.colors.accent : (props.colors.input ?? props.colors.raised),
      borderWidth: off ? 1 : 0,
      borderColor: "#2f3543",
    }}>
    <View style={{
      position: "absolute",
      width: 24,
      height: 18,
      borderRadius: 9,
      left: props.value ? 18 : 1,
      top: 1,
      backgroundColor: "#ffffff",
      boxShadow: "0 1px 1.75px #00000038",
    }} />
    {off
      ? <View style={{ position: "absolute", width: 6, height: 6, borderRadius: 3, borderWidth: 0.6, borderColor: "#ffffff", opacity: 0.75, left: 32, top: 7 }} />
      : null}
  </View>;
}

export function StatusDot(props: { colors: ApplicationTheme; tone?: "success" | "danger" | "muted" }) {
  const tone = props.tone ?? "success";
  const color = tone === "success" ? props.colors.success : tone === "danger" ? props.colors.danger : props.colors.muted;
  return <View style={{ width: 7, height: 7, borderRadius: 4, backgroundColor: color }} />;
}

/** A theme preview: the palette painted as a mini app, plus swatch circles. */
export function ThemeCard(props: { name: string; colors: ApplicationTheme; selected: boolean; active: boolean; captionColor?: string; onPress: () => void }) {
  const swatches = [props.colors.shell, props.colors.danger, "#c9b45a", "#7eae4d", props.colors.success, "#3aa6b9", props.colors.accent, "#6f7ff2", "#b07cff", "#d970b8", props.colors.muted];
  return <View onPress={props.onPress} role="button" accessibilityLabel={props.name} accessibilityState={{ selected: props.selected }} style={{ width: 182, gap: 9 }}>
    <View class="w-full overflow-hidden" style={{
      height: 74,
      borderRadius: 10,
      borderWidth: props.selected ? 2 : 1,
      borderColor: props.selected ? props.colors.accent : props.colors.border,
      backgroundColor: props.colors.background,
      paddingLeft: 11,
      paddingRight: 11,
      paddingTop: 10,
      paddingBottom: 9,
      justifyContent: "space-between",
    }}>
      <View class="flex-row items-center justify-between">
        <Text style={{ fontSize: 16, fontWeight: "600", color: props.colors.foreground }}>Aa</Text>
        <View class="flex-row items-center" style={{ gap: 5 }}>
          <View class="items-center justify-center" style={{ height: 19, paddingLeft: 5, paddingRight: 5, borderRadius: 5, backgroundColor: props.colors.dangerFill }}>
            <Text style={{ fontSize: 10, color: props.colors.danger }}>~/</Text>
          </View>
          <View style={{ width: 11, height: 19, borderRadius: 5, backgroundColor: props.colors.accent }} />
        </View>
      </View>
      <View class="flex-row items-center" style={{ gap: 4 }}>
        {swatches.map((swatch, index) => <View key={`${swatch}-${index}`} style={{ width: 9, height: 9, borderRadius: 5, backgroundColor: swatch }} />)}
      </View>
    </View>
    <Text style={{ fontSize: 12, fontWeight: props.selected ? "600" : "500", color: props.captionColor ?? (props.selected ? props.colors.foreground : props.colors.muted) }}>{props.name}</Text>
  </View>;
}
