<script setup lang="ts">
import type { CheckboxProps } from "@core/components/checkbox";

const props = withDefaults(defineProps<CheckboxProps>(), {
  disabled: false,
});

function toggle() {
  if (props.disabled) return;

  props.onCheckedChange?.(!props.checked);
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
    role="checkbox"
    :aria-checked="props.checked"
    :aria-label="props.ariaLabel"
    :disabled="props.disabled"
    :class="['checkbox', props.checked ? 'checkbox--checked' : '']"
    @click="toggle"
  >
    <span v-if="props.checked" aria-hidden="true" class="checkbox__indicator" />
  </button>
</template>
