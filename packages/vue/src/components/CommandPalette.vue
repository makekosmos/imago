<script setup lang="ts">
import { nextTick, ref, watch } from "vue";

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

watch(
  () => props.query,
  (value) => {
    if (value === undefined || value === query.value) {
      return;
    }

    query.value = value;
  },
);

watch(
  () => props.open,
  (value) => {
    if (!value) {
      return;
    }

    query.value = props.query ?? "";
    nextTick(() => inputRef.value?.focus());
  },
);

function close() {
  emit("update:open", false);
}

function updateQuery(value: string) {
  query.value = value;
  emit("update:query", value);
}

function handleInputKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
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
    <div class="command-palette__part-2" @click="close" />

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
