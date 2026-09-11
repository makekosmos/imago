<script setup lang="ts">
import type { Component } from "vue";

interface Props {
  icon: Component;
  label: string;
  active?: boolean;
  testId?: string;
  iconImage?: string;
  iconVariant?: "tile" | "plain";
  iconFrom?: string;
  iconTo?: string;
  iconWeight?: "thin" | "light" | "regular" | "bold" | "fill" | "duotone";
  iconOnly?: boolean;
}

withDefaults(defineProps<Props>(), {
  active: false,
  testId: undefined,
  iconImage: "",
  iconVariant: "plain",
  iconFrom: "var(--settings-sidebar-icon-from)",
  iconTo: "var(--settings-sidebar-icon-to)",
  iconWeight: "regular",
  iconOnly: false,
});

defineEmits<{
  click: [];
  contextmenu: [event: MouseEvent];
}>();
</script>

<template>
  <button
    type="button"
    class="kosmos-settings-sidebar-button settings-sidebar-button"
    :class="iconOnly ? 'settings-sidebar-button--is-state' : ''"
    :data-active="active ? 'true' : undefined"
    :data-testid="testId"
    :aria-current="active ? 'page' : undefined"
    @click="$emit('click')"
    @contextmenu="$emit('contextmenu', $event)"
  >
    <span
      class="settings-sidebar-button__part-2"
      :class="
        iconImage
          ? 'settings-sidebar-button--is-srgb'
          : iconVariant === 'plain'
            ? active
              ? 'settings-sidebar-button--is-srgb-2'
              : 'settings-sidebar-button--is-srgb-3'
            : 'settings-sidebar-button--is-state-2'
      "
      :style="{
        '--settings-sidebar-button-icon-from': iconFrom,
        '--settings-sidebar-button-icon-to': iconTo,
      }"
      aria-hidden="true"
    >
      <img v-if="iconImage" class="settings-sidebar-button__image" :src="iconImage" alt="" />
      <component v-else :is="icon" :size="16" :weight="iconWeight" :stroke-width="1.75" />
    </span>
    <span
      v-if="!iconOnly"
      class="kosmos-settings-sidebar-button__label settings-sidebar-button__part-4"
    >
      {{ label }}
    </span>
  </button>
</template>
