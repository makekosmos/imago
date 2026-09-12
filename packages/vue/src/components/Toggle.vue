<script setup lang="ts">
// Toggle / switch — бинарный control. Используется в settings
// Delphi/Eden для опций on/off.
//
// API совместим с v-model: `:model-value` / `@update:modelValue`.

import { computed } from "vue";

interface Props {
  modelValue: boolean;
  /** Текстовая подпись справа от switch'а (опционально — обычно label * приходит из обёртки SettingsRow). */ label?: string; disabled?: boolean; /** Аккессибильное имя для скрин-ридеров если нет label toggle__part-4'а. */
  ariaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const checked = computed(() => props.modelValue);

function toggle() {
  if (props.disabled) return;
  emit("update:modelValue", !checked.value);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === " " || e.key === "Enter") {
    e.preventDefault();
    toggle();
  }
}
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="checked"
    :aria-label="ariaLabel ?? label"
    :disabled="disabled"
    class="toggle"
    @click="toggle"
    @keydown="onKeydown"
  >
    <span
      :class="[
        'toggle--is-state',
        checked ? 'toggle--is-state-2' : '',
      ]"
    >
      <span
        :class="[
          'toggle--is-state-3',
          checked ? 'toggle--is-state-4' : '',
        ]"
      />
    </span>
    <span v-if="label" class="toggle__part-2">{{ label }}</span>
  </button>
</template>
