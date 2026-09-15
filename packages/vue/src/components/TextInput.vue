<script setup lang="ts">
import {
  textInputVariants,
  type TextInputProps,
} from "@core/components/textInput";

const props = withDefaults(
  defineProps<TextInputProps>(),
  {
    value: '',
    type: "text",
    size: "sm",
    disabled: false,
    readonly: false,
  },
);
</script>

<template>
  <input
    :value="props.value"
    :type="props.type"
    :placeholder="props.placeholder"
    :disabled="props.disabled"
    :autocomplete="props.autocomplete"
    :inputmode="props.inputmode"
    :aria-invalid="props.invalid"
    :data-invalid="props.invalid"
    :class="textInputVariants({
      size: props.size,
    })"
    @input="(e) =>props.onValueChange?.((e.target as HTMLInputElement).value)"
  />
</template>
