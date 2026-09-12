<script setup lang="ts">
import { computed, provide, useSlots } from "vue";
import Titlebar from "./Titlebar.vue";
import type { TitlebarPlatform } from "./types";

interface Props {
  appearance?: "default" | "settings";
  platform?: TitlebarPlatform;
  title?: string;
  titlebarTransparent?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  appearance: "default",
  platform: "windows",
  title: undefined,
  titlebarTransparent: false,
});

// Провайдим наличие сайдбара вниз по дереву, чтобы DesktopContentSurface
// мог автоматически убрать скругление верхнего-левого угла + левую границу,
// когда сайдбар не используется.
const slots = useSlots();
const hasSidebar = computed(() => Boolean(slots.sidebar));
provide("kosmosHasSidebar", hasSidebar);
</script>

<template>
  <div
    v-if="props.appearance === 'settings'"
    :class="[
      'kosmos-desktop-chrome-settings desktop-chrome--is-state',
      hasSidebar ? 'desktop-chrome--is-minmax' : 'desktop-chrome--is-state-2',
    ]"
  >
    <header
      :class="[
        'kosmos-desktop-chrome-settings__header desktop-chrome--is-state-3',
        // Native window controls: на macOS traffic lights слева → отступ
        // слева под них; на Windows min/max/close справа → отступ справа.
        props.platform === 'mac'
          ? 'desktop-chrome--is-px'
          : 'desktop-chrome--is-state-4',
      ]"
    >
      <div class="kosmos-desktop-chrome-settings__header-left desktop-chrome">
        <slot name="titlebar-leading" />
      </div>
      <div class="kosmos-desktop-chrome-settings__header-content desktop-chrome__part-2">
        <div class="kosmos-desktop-chrome-settings__header-content-leading desktop-chrome__part-3">
          <slot name="titlebar-content-leading" />
        </div>
        <div class="kosmos-desktop-chrome-settings__header-center desktop-chrome__part-4">
          <slot name="titlebar-center" />
        </div>
        <div class="kosmos-desktop-chrome-settings__header-right desktop-chrome__part-5">
          <slot name="titlebar-trailing" />
        </div>
      </div>
    </header>

    <div
      v-if="hasSidebar"
      class="kosmos-desktop-chrome-settings__sidebar-titlebar-divider desktop-chrome__part-6"
      aria-hidden="true"
    />

    <aside
      v-if="hasSidebar"
      class="desktop-chrome__part-7"
    >
      <slot name="sidebar" />
    </aside>

    <div
      :class="[
        'kosmos-desktop-chrome-settings__body desktop-chrome--is-state-5',
        hasSidebar ? 'desktop-chrome--is-start' : 'desktop-chrome--is-state-6',
      ]"
    >
      <slot />
    </div>
  </div>

  <div v-else class="desktop-chrome__part-8">
    <Titlebar
      :platform="props.platform"
      :title="props.title"
      :transparent="props.titlebarTransparent"
    >
      <template #leading>
        <slot name="titlebar-leading" />
      </template>
      <template #center>
        <slot name="titlebar-center" />
      </template>
      <template #trailing>
        <slot name="titlebar-trailing" />
      </template>
    </Titlebar>

    <div class="desktop-chrome__part-9">
      <aside v-if="hasSidebar" class="desktop-chrome__part-10">
        <slot name="sidebar" />
      </aside>

      <div class="desktop-chrome__part-11">
        <slot />
      </div>
    </div>
  </div>
</template>
