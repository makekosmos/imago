<script setup lang="ts">
import type { Component } from "vue";
import { RouterLink } from "vue-router";

const props = defineProps<{
  icon?: Component;
  iconImage?: string;
  to?: string;
  label?: string;
  active?: boolean;
  testId?: string;
}>();

const emit = defineEmits<{
  click: [];
}>();

function className(active: boolean) {
  const base =
    "kosmos-sidebar-btn sidebar-button__emit";
  return active
    ? `${base} sidebar-button--is-active`
    : base;
}
</script>

<template>
  <RouterLink v-if="to" :to="to" custom v-slot="{ href, navigate, isActive }">
    <a
      :href="href"
      :class="className(active ?? isActive)"
      :data-testid="testId"
      :title="label"
      @click="navigate"
    >
      <img v-if="iconImage" :src="iconImage" alt="" class="sidebar-button" />
      <component v-else-if="icon" :is="icon" :size="24" />
      <span v-if="label" class="sidebar-button__part-2">{{ label }}</span>
    </a>
  </RouterLink>
  <button
    v-else
    type="button"
    :class="className(active ?? false)"
    :data-testid="testId"
    :title="label"
    @click="emit('click')"
  >
    <img v-if="iconImage" :src="iconImage" alt="" class="sidebar-button__image" />
    <component v-else-if="icon" :is="icon" :size="24" />
    <span v-if="label" class="sidebar-button__part-4">{{ label }}</span>
  </button>
</template>
