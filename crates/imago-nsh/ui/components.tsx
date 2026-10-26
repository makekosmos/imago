import { effect, Image, Input, Scroll, Text, View } from "nsh";
import { Window as DesktopWindow, titleBar } from "nsh/desktop";
import { icons } from "./icons";
import type { IconName, ThemeColors, TitlebarAction } from "./types";
import { spacing, typography } from "./types";

export function Icon(props: { name: IconName; size?: number; color?: string }) {
  return <Image src={icons[props.name]} style={{ width: props.size ?? 16, height: props.size ?? 16, tintColor: props.color }} />;
}

export function NavigationItem(props: { label: string; icon: IconName; selected: boolean; colors: ThemeColors; onPress: () => void }) {
  return <View onPress={props.onPress} role="button" accessibilityState={{ selected: props.selected }} class="flex-row items-center gap-2 rounded-lg px-2.5" style={{ height: spacing.rowHeight, backgroundColor: props.selected ? props.colors.card : "transparent" }}>
    <Icon name={props.icon} color={props.colors.muted} />
    <Text class="text-[13px]" style={{ color: props.selected ? props.colors.foreground : props.colors.muted }}>{props.label}</Text>
  </View>;
}

export type SidebarEntry = { route: string; label: string; icon: IconName };
export type ProjectEntry = { id: string; title: string };

export function Sidebar(props: {
  visible: boolean;
  current: string;
  navigation: SidebarEntry[];
  otherPages: SidebarEntry[];
  projects: ProjectEntry[];
  moreOpen: boolean;
  colors: ThemeColors;
  onNavigate: (route: string) => void;
  onProject: (id: string) => void;
  onToggleMore: () => void;
}) {
  if (!props.visible) return null;
  const row = (item: SidebarEntry) => <NavigationItem key={item.route} label={item.label} icon={item.icon} selected={props.current === item.route} colors={props.colors} onPress={() => props.onNavigate(item.route)} />;
  return <View class="shrink-0 border-r px-2 pt-1" style={{ width: spacing.sidebarWidth, borderColor: props.colors.border }}>
    <View class="gap-0.5">{props.navigation.map(row)}</View>
    <View class="mt-5 rounded-lg px-2 pt-2" style={{ backgroundColor: props.colors.card }}>
      <View class="mb-1 h-8 flex-row items-center gap-2 px-0.5">
        <Icon name="project" color={props.colors.muted} />
        <Text class="flex-1 text-[13px]" style={{ color: props.colors.muted }}>Проекты</Text>
        <Icon name="plus" color={props.colors.muted} />
      </View>
      {props.projects.map((project) => <View key={project.id} onPress={() => props.onProject(project.id)} role="button" class="h-8 flex-row items-center gap-2">
        <View class="h-2 w-2 rounded-full" style={{ backgroundColor: props.colors.muted }} />
        <Text class="text-[13px]" style={{ color: props.colors.muted }}>{project.title}</Text>
      </View>)}
    </View>
    <View class="mt-auto mb-2">
      {props.moreOpen && <View class="mb-2 rounded-lg p-1" style={{ backgroundColor: props.colors.card }}>{props.otherPages.map(row)}</View>}
      <View onPress={props.onToggleMore} role="button" class="h-8 flex-row items-center gap-2 px-2.5">
        <Icon name="more" color={props.colors.muted} />
        <Text class="text-[13px]" style={{ color: props.colors.muted }}>Другое</Text>
      </View>
    </View>
  </View>;
}

export function Titlebar(props: { title: string; icon: IconName; colors: ThemeColors; onToggleSidebar: () => void; onAppearance: () => void; sidebarVisible?: boolean; showPageIdentity?: boolean; actions?: TitlebarAction[]; onAction?: (id: string) => void }) {
  effect(() => DesktopWindow.setTitleBar("hidden", spacing.titlebarHeight));
  const controls = titleBar();
  const customControls = controls.left === 0 && controls.right === 0;
  return <View class="w-full flex-row items-center" style={{ height: controls.height || spacing.titlebarHeight }} desktop={{ windowDrag: true }}>
    <View class="flex-row items-center" style={{ width: (props.sidebarVisible === false ? 40 : spacing.sidebarWidth) + controls.left, height: spacing.titlebarHeight }}>
      <View style={{ width: controls.left }} />
      <View onPress={props.onToggleSidebar} role="button" accessibilityLabel="Боковая панель" class="w-10 items-center justify-center" style={{ height: spacing.titlebarHeight }}>
        <Icon name="sidebar" color={props.colors.muted} />
      </View>
      <View class="flex-1" desktop={{ windowDrag: true }} />
    </View>
    {props.showPageIdentity === false ? <View class="flex-1" desktop={{ windowDrag: true }} /> : <View class="flex-1 flex-row items-center gap-1.5" desktop={{ windowDrag: true }}>
      <Icon name={props.icon} size={15} color={props.colors.muted} />
      <Text class="font-medium" style={{ fontSize: typography.titlebar, color: props.colors.foreground }}>{props.title}</Text>
    </View>}
    <View onPress={props.onAppearance} role="button" accessibilityLabel="Настройки отображения" class="w-10 items-center justify-center" style={{ height: spacing.titlebarHeight }}>
      <Icon name="display" color={props.colors.muted} />
    </View>
    {props.actions?.map((action) => <View key={action.id} onPress={() => props.onAction?.(action.id)} role="button" accessibilityLabel={action.label} class="w-10 items-center justify-center" style={{ height: spacing.titlebarHeight }}>
      <Icon name={action.icon} color={props.colors.muted} />
    </View>)}
    {customControls
      ? <View class="flex-row" style={{ width: 138, height: spacing.titlebarHeight }}>
          <View onPress={() => DesktopWindow.minimize()} role="button" accessibilityLabel="Свернуть окно" class="w-[46px] items-center justify-center" style={{ height: spacing.titlebarHeight }}><Icon name="windowMin" color={props.colors.foreground} size={14} /></View>
          <View onPress={() => DesktopWindow.toggleMaximize()} role="button" accessibilityLabel="Развернуть окно" class="w-[46px] items-center justify-center" style={{ height: spacing.titlebarHeight }}><Icon name="windowMax" color={props.colors.foreground} size={14} /></View>
          <View onPress={() => DesktopWindow.close()} role="button" accessibilityLabel="Закрыть окно" class="w-[46px] items-center justify-center" style={{ height: spacing.titlebarHeight }}><Icon name="windowClose" color={props.colors.foreground} size={14} /></View>
        </View>
      : <View style={{ width: controls.right }} />}
  </View>;
}

export function Button(props: { label: string; colors: ThemeColors; onPress: () => void; selected?: boolean; disabled?: boolean; className?: string; style?: Record<string, unknown>; textColor?: string }) {
  return <View onPress={props.disabled ? undefined : props.onPress} role="button" accessibilityState={{ selected: !!props.selected, disabled: !!props.disabled }} class={`rounded-md border px-3 py-2 ${props.className ?? ""}`} style={{ backgroundColor: props.selected ? props.colors.card : props.colors.background, borderColor: props.selected ? props.colors.accent : props.colors.border, opacity: props.disabled ? 0.5 : 1, ...props.style }}>
    <Text class="text-[13px]" style={{ color: props.textColor ?? (props.selected ? props.colors.foreground : props.colors.muted) }}>{props.label}</Text>
  </View>;
}

export function TextField(props: { value: string; placeholder: string; colors: ThemeColors; onChange: (value: string) => void; onSubmit?: (value: string) => void; accessibilityLabel?: string }) {
  return <Input value={props.value} onChange={props.onChange} onSubmit={props.onSubmit ? (event) => props.onSubmit?.(event.text) : undefined} accessibilityLabel={props.accessibilityLabel} placeholder={props.placeholder} class="h-10 rounded-lg px-3 text-[13px]" style={{ backgroundColor: props.colors.background, color: props.colors.foreground }} />;
}

export function ListRow(props: { colors: ThemeColors; children: any; onPress?: () => void }) {
  return <View onPress={props.onPress} role={props.onPress ? "button" : undefined} class="h-9 justify-center border-b" style={{ borderColor: props.colors.border }}>
    {props.children}
  </View>;
}

export function Dialog(props: { title: string; colors: ThemeColors; children: any; onDismiss?: () => void }) {
  return <View class="absolute inset-0 items-center justify-center bg-[#00000080] px-4">
    <View class="w-full max-w-[440px] rounded-xl border p-4" style={{ borderColor: props.colors.border, backgroundColor: props.colors.card }}>
      <Text class="mb-3 text-[14px]" style={{ color: props.colors.foreground }}>{props.title}</Text>
      {props.children}
    </View>
  </View>;
}

export function SectionTitle(props: { children: string; colors: ThemeColors }) {
  return <Text class="mb-3 text-[13px]" style={{ color: props.colors.muted }}>{props.children}</Text>;
}

export function Menu(props: { items: { label: string; destructive?: boolean; onPress: () => void }[]; colors: ThemeColors }) {
  return <View class="rounded-lg p-1" style={{ backgroundColor: props.colors.card }}>
    {props.items.map((item) => <View onPress={item.onPress} role="button" class="h-8 justify-center rounded-md px-2">
      <Text class="text-[13px]" style={{ color: item.destructive ? "#fb4934" : props.colors.foreground }}>{item.label}</Text>
    </View>)}
  </View>;
}

export function PageScroll(props: { colors: ThemeColors; children: any }) {
  return <Scroll class="min-h-0 flex-1 px-7 pt-4 pb-8" style={{ backgroundColor: props.colors.background }}>{props.children}</Scroll>;
}
