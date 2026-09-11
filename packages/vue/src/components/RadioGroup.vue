<script setup lang="ts" generic="T extends string | number">
// RadioGroup — вертикальный список радиокнопок с опциональным описанием
// под каждой опцией. Используется когда Dropdown слишком "сжимает" UX и
// хочется показать сразу все варианты с пояснениями.
//
// Стилистически — кастомные radio "пилюли" поверх токенов visuals,
// без `<input type=radio>` напрямую в UI (input спрятан для a11y).

import { computed } from "vue";

interface Option<V> {
  value: V;
  label: string;
  description?: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: T;
    options: ReadonlyArray<Option<T>>;
    /** Имя группы — для уникальности radio name (default — random). */
    name?: string;
    /** Layout. Default "vertical". */
    direction?: "vertical" | "horizontal";
    disabled?: boolean;
  }>(),
  {
    direction: "vertical",
    disabled: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [v: T];
}>();

const groupName = computed(
  () => props.name ?? `kosmos-rg-${Math.random().toString(36).slice(2, 8)}`,
);

function pick(opt: Option<T>) {
  if (props.disabled || opt.disabled) return;
  if (opt.value === props.modelValue) return;
  emit("update:modelValue", opt.value);
}
</script>

<template>
  <div
    :class="['radio-group--is-state', direction === 'horizontal' ? 'radio-group--is-col' : 'radio-group--is-state-2']"
    role="radiogroup"
  >
    <label
      v-for="opt in options"
      :key="String(opt.value)"
      :class="[
        'radio-group--is-state-3',
        opt.value === modelValue
          ? 'radio-group--is-state-4'
          : '',
        opt.disabled || disabled ? 'radio-group--is-state-5' : '',
        !opt.disabled && !disabled && opt.value !== modelValue
          ? 'radio-group--is-state-6'
          : '',
      ]"
    >
      <input
        type="radio"
        :name="groupName"
        :value="opt.value"
        :checked="opt.value === modelValue"
        :disabled="opt.disabled || disabled"
        class="radio-group"
        @change="pick(opt)"
      />
      <span
        :class="[
          'radio-group--is-state-7',
          opt.value === modelValue ? 'radio-group--is-state-8' : '',
        ]"
        aria-hidden="true"
      >
        <span
          :class="[
            'radio-group--is-state-9',
            opt.value === modelValue ? 'radio-group--is-scale' : 'radio-group--is-state-10',
          ]"
        />
      </span>
      <span class="radio-group__part-2">
        <span
          class="radio-group__part-3"
        >
          {{ opt.label }}
        </span>
        <span
          v-if="opt.description"
          class="radio-group__part-4"
        >
          {{ opt.description }}
        </span>
      </span>
    </label>
  </div>
</template>
