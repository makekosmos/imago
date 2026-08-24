<script setup lang="ts">
interface Props {
  className?: string;
}

defineProps<Props>();
</script>

<template>
  <div
    class="kosmos-settings-list flex flex-col overflow-hidden rounded-lg bg-[var(--settings-list-background)]"
    :class="className"
  >
    <slot />
  </div>
</template>

<style scoped>
.kosmos-settings-list :deep(> .settings-list__title) {
  margin: 0;
  padding: var(--space-2);
}
</style>
