<script setup lang="ts">
interface Props {
  /** Опасное действие — рендерится в --destructive тон. */
  destructive?: boolean;
  disabled?: boolean;
}

withDefaults(defineProps<Props>(), {
  destructive: false,
  disabled: false,
});

defineEmits<{
  click: [event: MouseEvent];
}>();
</script>

<template>
  <button
    type="button"
    role="menuitem"
    class="context-menu-item"
    :class="{
      'context-menu-item--is-destructive':
        destructive,
    }"
    :disabled="disabled"
    @click="(e) => $emit('click', e)"
  >
    <slot />
  </button>
</template>
