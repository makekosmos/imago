<script setup lang="ts">
import { onBeforeUnmount, useId, watch } from "vue";
import { isTopEscapeLayer, useEscapeLayer } from "../composables/useEscapeLayer";

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

const titleId = `kosmos-modal-title-${useId()}`;
const escapeToken = useEscapeLayer(() => props.open);

function onKey(e: KeyboardEvent) {
  if (e.key !== "Escape" || !props.open) return;
  if (!isTopEscapeLayer(escapeToken.value)) return;
  e.preventDefault();
  emit("close");
}

watch(
  () => props.open,
  (open) => {
    // immediate: модалка, открытая уже на маунте (v-if), тоже держит
    // escape-токен — без слушателя она глушила бы Escape для всех слоёв
    // ниже. document недоступен при SSR — тогда слушатели не нужны.
    if (typeof document === "undefined") return;
    if (open) {
      document.addEventListener("keydown", onKey, true);
    } else {
      document.removeEventListener("keydown", onKey, true);
    }
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  document.removeEventListener("keydown", onKey, true);
});

function onBackdropPointerDown(e: PointerEvent) {
  if (!props.closeOnBackdrop) return;
  if (e.button !== 0) return;
  // Не закрываемся по backdrop, пока над модалкой есть overlay
  // (dropdown, контекст-меню) — клик должен dismiss'ить только его.
  if (!isTopEscapeLayer(escapeToken.value)) return;
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
          :aria-labelledby="props.title ? titleId : undefined"
          :style="{ width: props.width ?? 'min(440px, 92vw)' }"
        >
          <header
            v-if="props.title || $slots.header"
            class="modal__header"
          >
            <slot name="header">
              <h2
                :id="titleId"
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
