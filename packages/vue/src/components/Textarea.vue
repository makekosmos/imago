<script setup lang="ts">
// Textarea — multi-line input primitive, визуально согласован с TextInput
// и Dropdown. Авто-resize отключён (controlled через `rows` + CSS resize).

import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    rows?: number;
    /** Минимальная высота в px (override CSS default). */
    minHeight?: number;
    /** CSS resize policy. */
    resize?: "none" | "vertical" | "horizontal" | "both";
    invalid?: boolean;
  }>(),
  {
    disabled: false,
    readonly: false,
    rows: 3,
    resize: "vertical",
    invalid: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [v: string];
  blur: [e: FocusEvent];
  focus: [e: FocusEvent];
}>();

const value = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

const normalizedMinHeight = computed(() =>
  props.minHeight ? Math.max(8, Math.round(props.minHeight / 8) * 8) : undefined,
);
</script>

<template>
  <textarea
    v-model="value"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    :rows="rows"
    :class="[
      'textarea--is-state',
      invalid ? 'textarea--is-state-2' : '',
    ]"
    :style="{
      resize,
      minHeight: normalizedMinHeight ? `${normalizedMinHeight}px` : undefined,
    }"
    @blur="(e) => emit('blur', e)"
    @focus="(e) => emit('focus', e)"
  />
</template>
