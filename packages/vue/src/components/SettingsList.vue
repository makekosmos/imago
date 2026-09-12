<script setup lang="ts">
interface Props {
  className?: string;
}

defineProps<Props>();
</script>

<template>
  <div
    class="kosmos-settings-list settings-list"
    :class="className"
  >
    <slot />
  </div>
</template>
