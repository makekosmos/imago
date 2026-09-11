<script setup lang="ts">
import type { Component } from "vue";

interface Props {
  icon: Component;
  title: string;
  description?: string;
  imageSrc?: string;
  iconFrom?: string;
  iconTo?: string;
}

withDefaults(defineProps<Props>(), {
  description: "",
  imageSrc: "",
  iconFrom: "var(--settings-sidebar-icon-from)",
  iconTo: "var(--settings-sidebar-icon-to)",
});
</script>

<template>
  <div class="settings-advanced-intro">
    <span
      class="settings-advanced-intro__part-2"
      :class="
        imageSrc
          ? 'settings-advanced-intro--is-srgb'
          : 'settings-advanced-intro--is-state'
      "
      :style="{
        '--settings-advanced-intro-icon-from': iconFrom,
        '--settings-advanced-intro-icon-to': iconTo,
      }"
      aria-hidden="true"
    >
      <img v-if="imageSrc" class="settings-advanced-intro__image" :src="imageSrc" alt="" />
      <component v-else :is="icon" :size="32" :stroke-width="2" />
    </span>
    <h1
      class="settings-advanced-intro__part-4"
    >
      {{ title }}
    </h1>
    <p
      v-if="description"
      class="settings-advanced-intro__part-5"
    >
      {{ description }}
    </p>
  </div>
</template>
