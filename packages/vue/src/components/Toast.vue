<script setup lang="ts">
import { computed } from "vue";
import { CheckCircle2, XCircle } from "@lucide/vue";
import type { ToastTone } from "../composables/useToast";

interface Props {
  message: string;
  title?: string;
  description?: string;
  tone?: ToastTone;
  loading?: boolean;
  /** Determinate progress 0–100; replaces the indeterminate loading bar. */
  progress?: number;
  closable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  tone: "info",
  loading: false,
  progress: undefined,
  closable: false,
});

const clampedProgress = computed(() =>
  props.progress === undefined ? undefined : Math.min(100, Math.max(0, props.progress)),
);

const toneIcon = computed(() =>
  props.tone === "success" ? CheckCircle2 : props.tone === "error" ? XCircle : null,
);

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
    <component
      v-else-if="toneIcon"
      :is="toneIcon"
      class="kosmos-toast__icon"
      :class="`kosmos-toast__icon--${tone}`"
      aria-hidden="true"
    />
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
    <div
      v-if="clampedProgress !== undefined"
      class="kosmos-toast__progressbar"
      role="progressbar"
      :aria-valuenow="clampedProgress"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <span
        class="kosmos-toast__progressbar-fill"
        :style="{ width: `${clampedProgress}%` }"
      />
    </div>
    <div
      v-else-if="loading"
      class="kosmos-toast__progress"
      aria-hidden="true"
    />
  </div>
</template>
