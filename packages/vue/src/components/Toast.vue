<script setup lang="ts">
import type { ToastTone } from "../composables/useToast";

interface Props {
  message: string;
  title?: string;
  description?: string;
  tone?: ToastTone;
  loading?: boolean;
  closable?: boolean;
}

withDefaults(defineProps<Props>(), {
  tone: "info",
  loading: false,
  closable: false,
});

const emit = defineEmits<{
  dismiss: [];
}>();
</script>

<template>
  <!-- Regression L6 (2026-05-24): aria-live lives on ToastHost (the polite
       live region). Double aria-live on a child inside a live region causes
       screen readers to announce twice. role="status" is kept for semantics. -->
  <div
    class="toast"
    :class="{
      'toast--is-success': tone === 'success',
      'toast--is-error':
        tone === 'error',
    }"
    role="status"
  >
    <div v-if="loading" class="kosmos-toast__spinner" aria-hidden="true" />
    <div class="toast__part-3">
      <div v-if="title" class="toast__part-4">{{ title }}</div>
      <div class="toast__part-5">{{ message }}</div>
      <div
        v-if="description"
        class="toast__part-6"
      >
        {{ description }}
      </div>
    </div>
    <button
      v-if="closable"
      type="button"
      class="toast__"
      aria-label="Закрыть уведомление"
      @click="emit('dismiss')"
    >
      ×
    </button>
    <div v-if="loading" class="kosmos-toast__progress" aria-hidden="true" />
  </div>
</template>
