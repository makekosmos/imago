<script setup lang="ts">
// IconButton — ghost icon button primitive с @kosmos/visuals tokens.
// Используется в titlebar'ах, mini-player widget'е,
// в context-aware controls. Заменяет ad-hoc `.iconbtn` / `.ctl-btn` /
// `.close-btn` CSS, которые тиражировались по экосистеме.

import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    /** Side length в px. Default 32, rounded to the nearest 8px grid step. */
    size?: number;
    /** Border-radius в px. Rounded to the nearest 8px grid step. */
    radius?: number;
    /** Tone влияет на hover-цвет:
        - default — нейтральный foreground hover;
        - destructive — destructive token на hover (для удаления / закрытия). */
    tone?: "default" | "destructive";
    /** Если true — оставляет область draggable (для `-webkit-app-region: drag`
        контекстов). По умолчанию false → область кнопки явно no-drag, иначе
        её нельзя кликнуть в drag-area mini-player'а. */
    draggable?: boolean;
    /** Button type — по умолчанию "button" (не submit). */
    type?: "button" | "submit";
    /** Disabled state. */
    disabled?: boolean;
  }>(),
  { size: 32, tone: "default", draggable: false, type: "button", disabled: false },
);

const buttonSize = computed(() => Math.max(8, Math.round(props.size / 8) * 8));
const buttonRadius = computed(() => {
  const requested = props.radius ?? 8;
  return Math.max(0, Math.round(requested / 8) * 8);
});
</script>

<template>
  <button
    :type="type"
    :disabled="disabled"
    :class="[
      'icon-button--is-state',
      tone === 'destructive'
        ? 'icon-button--is-state-2'
        : '',
      !draggable ? 'icon-button--is-state-3' : '',
    ]"
    :style="{
      '--icon-btn-size': `${buttonSize}px`,
      '--icon-btn-radius': `${buttonRadius}px`,
    }"
  >
    <slot />
  </button>
</template>
