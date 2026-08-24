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
    class="flex items-start gap-4 border-b border-[var(--border-color-low-emphasis)] bg-transparent px-4 py-4 last:border-b-0"
    :class="stacked ? 'flex-col' : ''"
  >
    <div v-if="$slots['leading-icon']" class="shrink-0" aria-hidden="true">
      <slot name="leading-icon" />
    </div>
    <div class="flex min-w-0 flex-col gap-1" :class="{ 'opacity-60': muted }">
      <div
        class="font-[var(--font-sans)] text-[length:var(--kosmos-text-body-size)] leading-[1.4] font-medium text-[var(--foreground)]"
      >
        {{ title }}
      </div>
      <div
        v-if="description"
        class="font-[var(--font-sans)] text-[length:var(--kosmos-text-caption-size)] leading-[1.4] font-medium text-[var(--muted-foreground)]"
      >
        {{ description }}
      </div>
    </div>
    <div class="flex shrink-0 items-start gap-2" :class="stacked ? 'w-full' : 'ml-auto'">
      <slot name="control">
        <slot />
      </slot>
    </div>
  </div>
</template>
