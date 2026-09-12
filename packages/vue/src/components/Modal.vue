<script setup lang="ts">
import { onBeforeUnmount, watch } from "vue";

interface Props {
  open: boolean;
  title?: string;
  /** Ширина модалки в CSS (по умолчанию `min(440px, 92vw)`). */
  width?: string;
  /** Закрытие по клику вне (на backdrop). По умолчанию true. */
  closeOnBackdrop?: boolean;
  /** Скрыть × кнопку в header'е (если у модалки есть явная "Отмена" в footer'е). */
  hideClose?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  width: undefined,
  closeOnBackdrop: true,
  hideClose: false,
});

const emit = defineEmits<{
  close: [];
}>();

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape" && props.open) emit("close");
}

watch(
  () => props.open,
  (open) => {
    if (open) {
      document.addEventListener("keydown", onKey);
    } else {
      document.removeEventListener("keydown", onKey);
    }
  },
);

onBeforeUnmount(() => {
  document.removeEventListener("keydown", onKey);
});

function onBackdropPointerDown(e: PointerEvent) {
  if (!props.closeOnBackdrop) return;
  if (e.target === e.currentTarget) emit("close");
}
</script>

<template>
  <Teleport to="body">
    <transition name="kosmos-modal">
      <div
        v-if="props.open"
        class="modal"
        role="presentation"
        @pointerdown="onBackdropPointerDown"
      >
        <div
          class="kosmos-modal__panel modal__part-2"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="props.title ? 'kosmos-modal-title' : undefined"
          :style="{ width: props.width ?? 'min(440px, 92vw)' }"
        >
          <header
            v-if="props.title || $slots.header"
            class="modal__header"
          >
            <slot name="header">
              <h2
                id="kosmos-modal-title"
                class="modal__part-4"
              >
                {{ props.title }}
              </h2>
            </slot>
            <button
              v-if="!props.hideClose"
              type="button"
              class="modal__"
              aria-label="Закрыть"
              @click="emit('close')"
            >
              ×
            </button>
          </header>
          <div class="modal__part-6">
            <slot />
          </div>
          <footer
            v-if="$slots.footer"
            class="modal__footer"
          >
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </transition>
  </Teleport>
</template>
