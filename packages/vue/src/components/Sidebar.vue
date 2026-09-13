<script setup lang="ts">
import { computed, shallowRef, onMounted, onUnmounted, useSlots, watch } from "vue";
import { RouterLink } from "vue-router";
import { ChevronRight, PanelLeftClose } from "@lucide/vue";
// eslint-disable-next-line import/no-unassigned-import
import "./sidebar.css";
import SidebarButton from "./SidebarButton.vue";
import type {
  SidebarConfig,
  SidebarNavItem,
  SidebarProjectItem,
  SidebarProjectGroup,
} from "./types";

interface Props {
  mode?: "navigation" | "panel";
  primaryItems?: SidebarNavItem[];
  tone?: "default" | "strong";
  title?: string;
  background?: string;
  projectItems?: SidebarProjectItem[];
  projectSectionLabel?: string;
  secondaryProjectItems?: SidebarProjectItem[];
  secondaryProjectSectionLabel?: string;
  projectGroups?: SidebarProjectGroup[];
  footerItems?: SidebarNavItem[];
  topItem?: SidebarNavItem;
  isMac?: boolean;
  className?: string;
  hidden?: boolean;
  offsetX?: number;
  toggleTitle?: string;
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
  toggleShortcut?: string;
  dragRegion?: boolean;
  initialConfig?: Partial<SidebarConfig>;
  showToggle?: boolean;
  reserveTopInset?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  mode: "navigation",
  primaryItems: () => [],
  tone: "default",
  title: undefined,
  background: undefined,
  projectItems: () => [],
  projectSectionLabel: "Проекты",
  secondaryProjectItems: () => [],
  secondaryProjectSectionLabel: "Ещё",
  projectGroups: () => [],
  footerItems: () => [],
  topItem: undefined,
  isMac: false,
  className: undefined,
  hidden: undefined,
  offsetX: 0,
  toggleTitle: "Скрыть сайдбар (⌘B)",
  defaultWidth: 240,
  minWidth: 160,
  maxWidth: 320,
  toggleShortcut: "meta+b|ctrl+b",
  dragRegion: true,
  initialConfig: undefined,
  showToggle: true,
  reserveTopInset: true,
});

const slots = useSlots();
const hasPanelTitleBar = computed(
  () => Boolean(props.title) || Boolean(slots["title-leading"]) || Boolean(slots["title-trailing"]),
);
const panelStyle = computed(() => ({
  "--kosmos-settings-sidebar-bg": props.background ?? "var(--sidebar-bg, var(--bg-app, #1d1d1f))",
}));

const emit = defineEmits<{
  configChange: [config: SidebarConfig];
  "update:hidden": [hidden: boolean];
}>();

const keyLayoutAliases: Record<string, string> = {
  q: "й",
  w: "ц",
  e: "у",
  r: "к",
  t: "е",
  y: "н",
  u: "г",
  i: "ш",
  o: "щ",
  p: "з",
  "[": "х",
  "]": "ъ",
  a: "ф",
  s: "ы",
  d: "в",
  f: "а",
  g: "п",
  h: "р",
  j: "о",
  k: "л",
  l: "д",
  ";": "ж",
  "'": "э",
  z: "я",
  x: "ч",
  c: "с",
  v: "м",
  b: "и",
  n: "т",
  m: "ь",
  ",": "б",
  ".": "ю",
};
const reverseKeyLayoutAliases: Record<string, string> = Object.fromEntries(
  Object.entries(keyLayoutAliases).map(([latinKey, localizedKey]) => [localizedKey, latinKey]),
);

const SIDEBAR_WIDTH_STEP = 8;

function snapSidebarWidth(value: number): number {
  return Math.round(value / SIDEBAR_WIDTH_STEP) * SIDEBAR_WIDTH_STEP;
}

function clampSidebarWidth(value: number): number {
  const snappedMin = snapSidebarWidth(props.minWidth);
  const snappedMax = snapSidebarWidth(props.maxWidth);
  return Math.max(snappedMin, Math.min(snappedMax, snapSidebarWidth(value)));
}

const width = shallowRef(clampSidebarWidth(props.initialConfig?.width ?? props.defaultWidth));
const _hidden = shallowRef(
  props.hidden !== undefined ? props.hidden : (props.initialConfig?.hidden ?? false),
);
const isResizing = shallowRef(false);
const animating = shallowRef(false);
const lineExpanded = shallowRef(false);
const collapsedGroups = shallowRef<Record<string, boolean>>({});
const sidebarShell = shallowRef<HTMLElement | null>(null);
const hoverHighlightVisible = shallowRef(false);
const hoverHighlightMoving = shallowRef(false);
const hoverHighlightStyle = shallowRef<Record<string, string>>({});

let mouseDownX = 0;
let mouseDownY = 0;

const resizeRaf = shallowRef<number | null>(null);
const animTimer = shallowRef<number | null>(null);
const isAnimatingRef = shallowRef(false);

const configRef = shallowRef<SidebarConfig>({
  width: width.value,
  hidden: _hidden.value,
});

const groupedProjectSections = computed<SidebarProjectGroup[]>(() => {
  if (props.projectGroups.length > 0) {
    return props.projectGroups.filter(
      (group) => group.items.length > 0 || Boolean(group.onAction),
    );
  }

  const groups: SidebarProjectGroup[] = [];

  if (props.projectItems.length > 0) {
    groups.push({
      id: "legacy-primary-project-group",
      label: props.projectSectionLabel,
      items: props.projectItems,
    });
  }

  if (props.secondaryProjectItems.length > 0) {
    groups.push({
      id: "legacy-secondary-project-group",
      label: props.secondaryProjectSectionLabel,
      items: props.secondaryProjectItems,
    });
  }

  return groups;
});

const hasProjectGroups = computed(() => groupedProjectSections.value.length > 0);
const hasTopBar = computed(
  () => props.showToggle || Boolean(props.topItem) || Boolean(props.title),
);
const shellClasses = computed(() => [
  "kosmos-sidebar-shell",
  "sidebar__has-top-bar",
  { "kosmos-sidebar-shell--mac-safe-top": props.isMac && props.reserveTopInset },
  { "kosmos-sidebar-shell--with-top-bar": hasTopBar.value },
  { "kosmos-sidebar-shell--drag-region": props.dragRegion },
]);

const topToggleClass =
  "kosmos-sidebar-top-toggle sidebar__top-toggle";
const groupHeaderClass =
  "sidebar__group-header";
const groupActionClass =
  "sidebar__group-action";
const projectLinkBaseClass =
  "kosmos-sidebar-project-link widget-nav-item sidebar__project-link-base";
const projectLinkActiveClass =
  "sidebar__project-link-active";

watch([width, _hidden], () => {
  configRef.value = { width: width.value, hidden: _hidden.value };
});

watch(
  groupedProjectSections,
  (groups) => {
    const nextState: Record<string, boolean> = {};
    for (const group of groups) {
      nextState[group.id] = collapsedGroups.value[group.id] ?? !!group.defaultCollapsed;
    }
    collapsedGroups.value = nextState;
  },
  { immediate: true, deep: true },
);

watch(
  () => props.hidden,
  (val) => {
    if (val !== undefined && val !== _hidden.value) {
      startAnimation();
      _hidden.value = val;
    }
  },
);

function notifyConfigChange(config: SidebarConfig) {
  emit("configChange", config);
}

function startAnimation() {
  if (animTimer.value) {
    window.clearTimeout(animTimer.value);
  }
  isAnimatingRef.value = true;
  animating.value = true;
  animTimer.value = window.setTimeout(() => {
    isAnimatingRef.value = false;
    animating.value = false;
    animTimer.value = null;
  }, 330);
}

function toggle() {
  startAnimation();
  _hidden.value = !_hidden.value;
  emit("update:hidden", _hidden.value);
  notifyConfigChange({ width: width.value, hidden: _hidden.value });
}

function expandKeyVariants(key: string | null | undefined): Set<string> {
  const variants = new Set<string>();
  if (!key) return variants;

  const normalized = key.toLowerCase();
  variants.add(normalized);

  const alias = keyLayoutAliases[normalized];
  if (alias) {
    variants.add(alias);
  }

  const reverseAlias = reverseKeyLayoutAliases[normalized];
  if (reverseAlias) {
    variants.add(reverseAlias);
  }

  return variants;
}

function matchesShortcut(e: KeyboardEvent, shortcut: string): boolean {
  const parts = shortcut.toLowerCase().split("+");
  const key = parts[parts.length - 1];
  const needsMeta = parts.includes("meta");
  const needsCtrl = parts.includes("ctrl");
  const needsShift = parts.includes("shift");
  const needsAlt = parts.includes("alt");

  if (needsMeta && !e.metaKey) return false;
  if (needsCtrl && !e.ctrlKey) return false;
  if (needsShift && !e.shiftKey) return false;
  if (needsAlt && !e.altKey) return false;

  const shortcutKeys = expandKeyVariants(key);
  const eventKeys = expandKeyVariants(e.key);
  const codeKey = e.code.startsWith("Key") ? e.code.slice(3).toLowerCase() : null;
  for (const codeVariant of expandKeyVariants(codeKey)) {
    eventKeys.add(codeVariant);
  }

  return [...shortcutKeys].some((shortcutKey) => eventKeys.has(shortcutKey));
}

function handleKeydown(e: KeyboardEvent) {
  if (props.mode !== "navigation" || !props.toggleShortcut) return;

  const shortcuts = props.toggleShortcut.split("|");
  if (!shortcuts.some((shortcut) => matchesShortcut(e, shortcut))) return;

  e.preventDefault();
  toggle();
}

function handleResizeStart(e: MouseEvent) {
  e.preventDefault();
  mouseDownX = e.clientX;
  mouseDownY = e.clientY;
  isResizing.value = true;
  document.body.classList.add("sidebar-resizing");
}

function handleResizeMove(e: MouseEvent) {
  if (isAnimatingRef.value || _hidden.value) return;

  if (resizeRaf.value) {
    cancelAnimationFrame(resizeRaf.value);
  }

  resizeRaf.value = requestAnimationFrame(() => {
    if (isAnimatingRef.value) return;

    const newWidth = e.clientX - (props.offsetX ?? 0);
    const clamped = clampSidebarWidth(newWidth);
    width.value = clamped;
    configRef.value = { ...configRef.value, width: clamped };
  });
}

function handleResizeEnd(e: MouseEvent) {
  if (resizeRaf.value) {
    cancelAnimationFrame(resizeRaf.value);
    resizeRaf.value = null;
  }
  isResizing.value = false;
  document.body.classList.remove("sidebar-resizing");

  const dx = Math.abs(e.clientX - mouseDownX);
  const dy = Math.abs(e.clientY - mouseDownY);
  if (dx < 4 && dy < 4) {
    lineExpanded.value = true;
    window.setTimeout(() => {
      lineExpanded.value = false;
      toggle();
    }, 180);
    return;
  }

  notifyConfigChange(configRef.value);
}

function isGroupCollapsed(groupId: string): boolean {
  return collapsedGroups.value[groupId] ?? false;
}

function toggleGroup(groupId: string) {
  collapsedGroups.value = {
    ...collapsedGroups.value,
    [groupId]: !isGroupCollapsed(groupId),
  };
}

function showHoverHighlight(event: PointerEvent | FocusEvent) {
  const target = event.target instanceof Element
    ? event.target.closest<HTMLElement>(".sidebar-button__emit, .sidebar__project-link-base")
    : null;
  if (!target) {
    hideHoverHighlight();
    return;
  }
  if (!sidebarShell.value) return;

  const shellRect = sidebarShell.value.getBoundingClientRect();
  const targetRect = target.getBoundingClientRect();
  hoverHighlightMoving.value = hoverHighlightVisible.value;
  hoverHighlightStyle.value = {
    width: `${targetRect.width}px`,
    height: `${targetRect.height}px`,
    transform: `translate3d(${targetRect.left - shellRect.left}px, ${targetRect.top - shellRect.top}px, 0)`,
  };
  hoverHighlightVisible.value = true;
}

function hideHoverHighlight() {
  hoverHighlightVisible.value = false;
  hoverHighlightMoving.value = false;
}

watch(isResizing, (resizing) => {
  if (resizing) {
    window.addEventListener("mousemove", handleResizeMove);
    window.addEventListener("mouseup", handleResizeEnd);
  } else {
    window.removeEventListener("mousemove", handleResizeMove);
    window.removeEventListener("mouseup", handleResizeEnd);
  }
});

onMounted(() => {
  if (props.mode === "navigation") window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("mousemove", handleResizeMove);
  window.removeEventListener("mouseup", handleResizeEnd);
  if (animTimer.value) window.clearTimeout(animTimer.value);
  if (resizeRaf.value) cancelAnimationFrame(resizeRaf.value);
  document.body.classList.remove("sidebar-resizing");
});

const wrapperStyle = computed(() => {
  const background = props.background ? { "--sidebar-bg": props.background } : {};
  if (_hidden.value) return { width: "0px", ...background };
  return { width: `${width.value}px`, ...background };
});

const wrapperClasses = computed(() =>
  [
    "kosmos-sidebar-wrapper",
    "sidebar__wrapper-classes",
    _hidden.value ? "collapsed sidebar--is-value" : "",
    animating.value ? "animating" : "",
    isResizing.value ? "is-resizing" : "",
    props.className ?? "",
  ]
    .filter(Boolean)
    .join(" "),
);
</script>

<template>
  <aside
    v-if="props.mode === 'panel'"
    class="kosmos-settings-sidebar sidebar"
    :data-tone="props.tone"
    :style="panelStyle"
  >
    <div
      v-if="hasPanelTitleBar"
      class="kosmos-settings-sidebar__title sidebar__part-2"
    >
      <div
        v-if="$slots['title-leading']"
        class="sidebar__part-3"
      >
        <slot name="title-leading" />
      </div>
      <span v-if="props.title" class="sidebar__part-4">{{ props.title }}</span>
      <div
        v-if="$slots['title-trailing']"
        class="sidebar__part-5"
      >
        <slot name="title-trailing" />
      </div>
    </div>
    <div
      class="kosmos-settings-sidebar__content sidebar__part-6"
    >
      <slot />
    </div>
  </aside>

  <div v-else :class="wrapperClasses" :style="wrapperStyle" data-testid="kosmos-sidebar">
    <div
      class="kosmos-sidebar-content sidebar__part-7"
    >
      <aside
        v-if="!_hidden"
        ref="sidebarShell"
        :class="shellClasses"
        @pointerover="showHoverHighlight"
        @pointerleave="hideHoverHighlight"
        @focusin="showHoverHighlight"
        @focusout="hideHoverHighlight"
        @scroll.capture="hideHoverHighlight"
      >
        <div
          aria-hidden="true"
          :class="[
            'kosmos-sidebar-hover-highlight',
            { 'is-visible': hoverHighlightVisible, 'is-moving': hoverHighlightMoving },
          ]"
          :style="[{ pointerEvents: 'none' }, hoverHighlightStyle]"
        />
        <div
          v-if="hasTopBar"
          :class="[
            'kosmos-sidebar-top sidebar--is-state',
            { 'kosmos-sidebar-top--titled': props.title },
          ]"
        >
          <span v-if="props.title" class="kosmos-sidebar-title sidebar__part-8">
            {{ props.title }}
          </span>
          <button
            v-if="props.topItem"
            type="button"
            :class="topToggleClass"
            :data-testid="props.topItem.testId"
            :title="props.topItem.label"
            @click="
              'onClick' in props.topItem && typeof props.topItem.onClick === 'function'
                ? props.topItem.onClick()
                : undefined
            "
          >
            <component :is="props.topItem.icon" :size="24" />
          </button>
          <button
            v-if="props.showToggle"
            type="button"
            :class="topToggleClass"
            data-testid="sidebar-toggle"
            :title="toggleTitle"
            @click="toggle"
          >
            <PanelLeftClose :size="24" />
          </button>
        </div>

        <div
          class="kosmos-sidebar-body sidebar__part-9"
        >
          <SidebarButton
            v-for="item in primaryItems"
            :key="item.id"
            :icon="item.icon"
            :to="item.to"
            :label="item.label"
            :active="item.active"
            :test-id="item.testId"
            @click="
              'onClick' in item && typeof item.onClick === 'function' ? item.onClick() : undefined
            "
          />

          <slot />

          <template v-if="hasProjectGroups">
            <div class="sidebar__part-10">
              <section v-for="group in groupedProjectSections" :key="group.id" class="sidebar__part-11">
                <div class="sidebar__part-12">
                  <button
                    type="button"
                    :class="groupHeaderClass"
                    :data-testid="`sidebar-group-${group.id}`"
                    @click="toggleGroup(group.id)"
                  >
                    <ChevronRight
                      :size="16"
                      :class="[
                        'sidebar--is-state-2',
                        isGroupCollapsed(group.id) ? '' : 'sidebar--is-state-3',
                      ]"
                    />
                    <span>{{ group.label }}</span>
                  </button>

                  <button
                    v-if="group.actionIcon && group.onAction"
                    type="button"
                    :class="groupActionClass"
                    :title="group.actionLabel"
                    :data-testid="group.actionTestId"
                    @click="group.onAction()"
                  >
                    <component :is="group.actionIcon" :size="16" />
                  </button>
                </div>

                <div
                  v-if="!isGroupCollapsed(group.id) && group.items.length > 0"
                  class="sidebar__part-13"
                >
                  <template v-for="project in group.items" :key="project.id">
                    <RouterLink
                      v-if="project.to"
                      :to="project.to"
                      :data-testid="project.testId"
                      :class="[projectLinkBaseClass, project.active ? projectLinkActiveClass : '']"
                      @contextmenu="project.onContextMenu?.($event)"
                    >
                      <span
                        v-if="project.iconSrc"
                        class="kosmos-sidebar-project-icon-wrap sidebar__part-14"
                        :style="{
                          '--kosmos-project-icon-color':
                            project.iconColor ?? project.color ?? 'var(--muted-foreground)',
                        }"
                      >
                        <span
                          class="kosmos-sidebar-project-icon sidebar__part-15"
                          :style="{ '--kosmos-project-icon-src': `url(${project.iconSrc})` }"
                          aria-hidden="true"
                        />
                      </span>
                      <span
                        v-else
                        :class="[
                          'kosmos-sidebar-project-dot',
                          'sidebar--is-state-4',
                          project.colorClass ?? 'sidebar--is-state-5',
                        ]"
                        :style="project.color ? { backgroundColor: project.color } : undefined"
                      />
                      <span class="sidebar__part-16">{{ project.label }}</span>
                    </RouterLink>

                    <button
                      v-else
                      type="button"
                      :data-testid="project.testId"
                      :class="[projectLinkBaseClass, project.active ? projectLinkActiveClass : '']"
                      @click="
                        'onClick' in project && typeof project.onClick === 'function'
                          ? project.onClick()
                          : undefined
                      "
                      @contextmenu="project.onContextMenu?.($event)"
                    >
                      <span
                        v-if="project.iconSrc"
                        class="kosmos-sidebar-project-icon-wrap sidebar__part-17"
                        :style="{
                          '--kosmos-project-icon-color':
                            project.iconColor ?? project.color ?? 'var(--muted-foreground)',
                        }"
                      >
                        <span
                          class="kosmos-sidebar-project-icon sidebar__part-18"
                          :style="{ '--kosmos-project-icon-src': `url(${project.iconSrc})` }"
                          aria-hidden="true"
                        />
                      </span>
                      <span
                        v-else
                        :class="[
                          'kosmos-sidebar-project-dot',
                          'sidebar--is-state-4',
                          project.colorClass ?? 'sidebar--is-state-5',
                        ]"
                        :style="project.color ? { backgroundColor: project.color } : undefined"
                      />
                      <span class="sidebar__part-19">{{ project.label }}</span>
                    </button>
                  </template>
                </div>
              </section>
            </div>
          </template>
        </div>

        <div v-if="footerItems.length > 0" class="sidebar__part-20">
          <SidebarButton
            v-for="item in footerItems"
            :key="item.id"
            :icon="item.icon"
            :to="item.to"
            :label="item.label"
            :active="item.active"
            :test-id="item.testId"
            @click="
              'onClick' in item && typeof item.onClick === 'function' ? item.onClick() : undefined
            "
          />
        </div>
      </aside>
    </div>

    <div
      v-if="!_hidden"
      class="kosmos-sidebar-resize-handle sidebar__part-21"
      data-testid="kosmos-sidebar-resize-handle"
      @mousedown="handleResizeStart"
    >
      <div
        :class="[
          'kosmos-resize-handle-line sidebar--is-state-6',
          lineExpanded ? 'expanded' : '',
        ]"
      />
    </div>
  </div>
</template>
