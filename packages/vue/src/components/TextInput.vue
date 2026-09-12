<script setup lang="ts">
// TextInput — single-line input primitive, визуально согласован с Dropdown
// (та же высота, бордеры, hover/focus). Поддерживает type=text/password/email/url
// и события v-model.

import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    type?: "text" | "password" | "email" | "url" | "search";
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    /** Размер: md (default) / sm. */
    size?: "md" | "sm";
    /** Полная ширина контейнера. */
    block?: boolean;
    /** Visual error / invalid state. */
    invalid?: boolean;
    /** Автокомплит (для password — "current-password" / "new-password"). */
    autocomplete?: string;
    /** Inputmode для виртуальных клавиатур. */
    inputmode?: "text" | "numeric" | "decimal" | "tel" | "search" | "email" | "url";
  }>(),
  {
    type: "text",
    disabled: false,
    readonly: false,
    size: "sm",
    block: true,
    invalid: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [v: string];
  blur: [e: FocusEvent];
  focus: [e: FocusEvent];
  keydown: [e: KeyboardEvent];
}>();

const value = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});
</script>

<template>
  <input
    v-model="value"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    :autocomplete="autocomplete"
    :inputmode="inputmode"
    :class="[
      'text-input',
      size === 'sm' ? 'text-input-sm' : 'text-input-md',
      block ? 'text-input-block' : '',
      invalid ? 'text-input-invalid' : '',
    ]"
    @blur="(e) => emit('blur', e)"
    @focus="(e) => emit('focus', e)"
    @keydown="(e) => emit('keydown', e)"
  />
</template>
