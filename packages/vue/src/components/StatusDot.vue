<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import type { StatusDotTone } from "./types";

interface Props {
  tone?: StatusDotTone;
  label: string;
  sideOffset?: number;
  showLabel?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  tone: "neutral",
  sideOffset: 8,
  showLabel: false,
});

const rootRef = ref<HTMLElement | null>(null);
const open = ref(false);

const toneClass = computed(() => {
  switch (props.tone) {
    case "success":
      return "status-dot__root-ref-2";
    case "warning":
      return "status-dot__open";
    case "danger":
      return "status-dot__tone";
    case "neutral":
    default:
      return "status-dot__part-4-2";
  }
});

const normalizedSideOffset = computed(() => Math.max(0, Math.round(props.sideOffset / 8) * 8));

function toggle() {
  open.value = !open.value;
}

function close() {
  open.value = false;
}

function handleDocumentPointerDown(event: PointerEvent) {
  if (!open.value || !rootRef.value) {
    return;
  }

  const target = event.target;
  if (target instanceof Node && rootRef.value.contains(target)) {
    return;
  }

  close();
}

function handleDocumentKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    close();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", handleDocumentPointerDown);
  document.addEventListener("keydown", handleDocumentKeydown);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", handleDocumentPointerDown);
  document.removeEventListener("keydown", handleDocumentKeydown);
});
</script>

<template>
  <div ref="rootRef" class="status-dot__root-ref" :class="showLabel ? 'status-dot--is-state' : ''">
    <button
      type="button"
      class="status-dot__button"
      :class="toneClass"
      :title="label"
      :aria-label="label"
      :aria-expanded="open"
      @click="toggle()"
    >
      <span class="status-dot__part-3" />
    </button>
    <span
      v-if="showLabel"
      class="status-dot__part-4"
    >
      {{ label }}
    </span>

    <div
      v-if="open"
      class="status-dot__part-5"
      :style="{ top: `calc(100% + ${normalizedSideOffset}px)` }"
      role="dialog"
      :aria-label="label"
    >
      <slot>
        <p class="status-dot__part-6">{{ label }}</p>
      </slot>
    </div>
  </div>
</template>
