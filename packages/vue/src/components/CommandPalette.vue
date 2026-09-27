<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import { isTopEscapeLayer, useEscapeLayer } from "../composables/useEscapeLayer";

const props = defineProps<{
  open: boolean;
  placeholder?: string;
  query?: string;
  dialogTestId?: string;
  inputTestId?: string;
}>();

const emit = defineEmits<{
  "update:open": [boolean];
  "update:query": [string];
}>();

const query = ref(props.query ?? "");
const inputRef = ref<HTMLInputElement>();
const listRef = ref<HTMLElement>();

// Занимаем слой в стеке Escape-overlay'ей, пока палитра открыта — нижележащие
// слои (модалка) не должны перехватывать её Escape. Вложенные overlay
// (dropdown в результатах) закрываются сами и ставят e.defaultPrevented.
const escapeToken = useEscapeLayer(() => props.open);

watch(
  () => props.query,
  (value) => {
    if (value === undefined || value === query.value) {
      return;
    }

    query.value = value;
  },
);

// Escape на document capture: закрываемся при любом фокусе внутри палитры
// (не только на input/list), но только когда палитра — верхний слой.
function onDocEscape(e: KeyboardEvent) {
  if (e.key !== "Escape" || !props.open) return;
  if (!isTopEscapeLayer(escapeToken.value)) return;
  e.preventDefault();
  close();
}

watch(
  () => props.open,
  (value) => {
    if (value) {
      query.value = props.query ?? "";
      nextTick(() => inputRef.value?.focus());
    }
    // immediate: палитра, открытая на маунте, держит escape-токен — без
    // слушателя она глушила бы Escape у слоёв ниже. SSR: document нет.
    if (typeof document === "undefined") return;
    if (value) {
      document.addEventListener("keydown", onDocEscape, true);
    } else {
      document.removeEventListener("keydown", onDocEscape, true);
    }
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  document.removeEventListener("keydown", onDocEscape, true);
});

function close() {
  emit("update:open", false);
}

function onBackdropPointerDown(e: PointerEvent) {
  if (e.button !== 0) return;
  // Клик по backdrop при открытом внутреннем overlay dismiss'ит только его.
  if (!isTopEscapeLayer(escapeToken.value)) return;
  close();
}

function updateQuery(value: string) {
  query.value = value;
  emit("update:query", value);
}

function handleInputKeydown(event: KeyboardEvent) {
  if (event.defaultPrevented) return;
  if (event.key === "Escape") {
    event.preventDefault();
    close();
    return;
  }

  if (event.key === "ArrowDown") {
    event.preventDefault();
    const items = listRef.value?.querySelectorAll<HTMLElement>("[data-cmd-item]");
    if (items?.length) {
      items[0].focus();
    }
  }
}

function handleListKeydown(event: KeyboardEvent) {
  if (event.defaultPrevented) return;
  const items = [...(listRef.value?.querySelectorAll<HTMLElement>("[data-cmd-item]") ?? [])];
  const index = items.findIndex((item) => item === document.activeElement);

  if (event.key === "ArrowDown") {
    event.preventDefault();
    items[(index + 1) % items.length]?.focus();
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    if (index <= 0) {
      inputRef.value?.focus();
    } else {
      items[index - 1].focus();
    }
    return;
  }

  if (event.key === "Escape") {
    event.preventDefault();
    close();
    return;
  }

  if (event.key.length === 1) {
    inputRef.value?.focus();
  }
}
</script>

<template>
  <div v-if="open" class="command-palette">
    <div class="command-palette__part-2" @pointerdown="onBackdropPointerDown" />

    <div
      class="command-palette__part-3"
      style="background: var(--color-shape-highlight-light-solid)"
      :data-testid="dialogTestId"
    >
      <div class="command-palette__part-4">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="command-palette__icon"
        >
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.3-4.3" />
        </svg>
        <input
          ref="inputRef"
          :value="query"
          :placeholder="placeholder ?? 'Поиск...'"
          class="command-palette__input-ref"
          :data-testid="inputTestId"
          @input="updateQuery(($event.target as HTMLInputElement).value)"
          @keydown="handleInputKeydown"
        />
        <kbd
          class="command-palette__part-7"
        >
          esc
        </kbd>
      </div>

      <div class="command-palette__part-8" />

      <div ref="listRef" class="command-palette__list-ref" @keydown="handleListKeydown">
        <slot :query="query" :close="close" />
      </div>
    </div>
  </div>
</template>
