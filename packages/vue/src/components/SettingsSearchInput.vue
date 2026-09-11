<script setup lang="ts">
import { Search } from "@lucide/vue";

interface Props {
  modelValue: string;
  placeholder?: string;
}

withDefaults(defineProps<Props>(), {
  placeholder: "Поиск",
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

function onInput(event: Event) {
  if (event.target instanceof HTMLInputElement) {
    emit("update:modelValue", event.target.value);
  }
}
</script>

<template>
  <label
    class="settings-search-input"
  >
    <Search
      class="settings-search-input__part-2"
      :size="14"
      :stroke-width="2"
      aria-hidden="true"
    />
    <input
      class="settings-search-input__input"
      :value="modelValue"
      :placeholder="placeholder"
      type="text"
      @input="onInput"
    />
  </label>
</template>
