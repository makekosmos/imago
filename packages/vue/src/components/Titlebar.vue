<script setup lang="ts">
import { computed } from "vue";
import type { TitlebarPlatform } from "./types";

interface Props {
  platform?: TitlebarPlatform;
  title?: string;
  transparent?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  platform: "windows",
  title: undefined,
  transparent: false,
});

const titlebarClasses = computed(() => [
  "kosmos-titlebar titlebar__props",
  `kosmos-titlebar--${props.platform}`,
  props.transparent ? "titlebar--is-transparent" : "titlebar__part-4-2",
]);
</script>

<template>
  <header :class="titlebarClasses">
    <div class="titlebar">
      <slot name="leading" />
    </div>

    <div class="titlebar__part-2">
      <slot name="center">
        <span
          v-if="title"
          class="titlebar__part-3"
          >{{ title }}</span
        >
      </slot>
    </div>

    <div class="titlebar__part-4">
      <slot name="trailing" />
    </div>
  </header>
</template>
