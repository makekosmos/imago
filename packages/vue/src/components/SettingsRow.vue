<script setup lang="ts">
// SettingsRow — стандартная строка в Settings UI: title + description слева,
// action / value control справа. Используется в Eden / Delphi /
// Arrancador settings.
//
// Slot `control` — место для Toggle, select, button, input, etc.
// Slot `default` — fallback alternative для control.

interface Props {
  title: string;
  description?: string;
  /** Когда true — строка визуально выглядит disabled (но control сам
   * управляет своим disabled-state). */
  muted?: boolean;
  stacked?: boolean;
}

withDefaults(defineProps<Props>(), {
  muted: false,
  stacked: false,
});
</script>

<template>
  <div
    class="settings-row"
    :class="stacked ? 'settings-row--is-state' : ''"
  >
    <div v-if="$slots['leading-icon']" class="settings-row__part-2" aria-hidden="true">
      <slot name="leading-icon" />
    </div>
    <div class="settings-row__part-3" :class="{ 'settings-row--is-muted': muted }">
      <div
        class="settings-row__part-4"
      >
        {{ title }}
      </div>
      <div
        v-if="description"
        class="settings-row__part-5"
      >
        {{ description }}
      </div>
    </div>
    <div
      v-if="$slots.control || $slots.default"
      class="settings-row__part-6"
      :class="stacked ? 'settings-row--is-auto' : 'settings-row--is-state-2'"
    >
      <slot name="control">
        <slot />
      </slot>
    </div>
  </div>
</template>
