<script setup lang="ts">
defineOptions({
  inheritAttrs: false,
});

const props = defineProps<{
  class?: string;
}>();
</script>

<template>
  <div
    v-bind="$attrs"
    :class="[
      'kosmos-skeleton skeleton--is-state',
      props.class,
    ]"
    aria-hidden="true"
  />
</template>
