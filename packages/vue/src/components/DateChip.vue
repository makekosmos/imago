<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef } from "vue";
import { Calendar as CalendarIcon, X } from "@lucide/vue";
import Calendar from "./Calendar.vue";

interface Props {
  /** ISO `YYYY-MM-DD` или null. */
  value: string | null;
  /** Текст когда дата не выбрана. */
  placeholder?: string;
  /** Если true — компактный размер чипа. */
  compact?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: "Дата",
  compact: false,
});

const emit = defineEmits<{
  "update:value": [iso: string | null];
}>();

const open = shallowRef(false);
const anchorRef = ref<HTMLElement | null>(null);

const RU_MONTHS_SHORT = [
  "янв",
  "фев",
  "мар",
  "апр",
  "май",
  "июн",
  "июл",
  "авг",
  "сен",
  "окт",
  "ноя",
  "дек",
] as const;

const label = computed(() => {
  if (!props.value) return props.placeholder;
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(props.value);
  if (!m) return props.value;
  const day = Number(m[3]);
  const month = RU_MONTHS_SHORT[Number(m[2]) - 1];
  return `${day} ${month}`;
});

const todayIso = computed(() => {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
});

function pick(iso: string) {
  emit("update:value", iso);
  open.value = false;
}

function clear(e: MouseEvent) {
  e.stopPropagation();
  emit("update:value", null);
}

function toggle() {
  open.value = !open.value;
}

function onDocPointerDown(e: PointerEvent) {
  if (!open.value || !anchorRef.value) return;
  if (e.target instanceof Node && anchorRef.value.contains(e.target)) return;
  open.value = false;
}

function onDocKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape" && open.value) open.value = false;
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointerDown);
  document.addEventListener("keydown", onDocKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointerDown);
  document.removeEventListener("keydown", onDocKeyDown);
});
</script>

<template>
  <div ref="anchorRef" class="date-chip__anchor-ref">
    <!--
      Composite chip = 2 sibling button'а в одном flex-контейнере. Раньше
      clear был `<span role="button">` внутри `<button>` — невалидный
      nested interactive, screen reader'ы collapse'или в одну кнопку и
      «Очистить дату» становилась недоступна с клавиатуры. Теперь две
      нормальные кнопки, контейнер только визуально объединяет их.
    -->
    <div
      :class="[
        'date-chip--is-state',
        props.value
          ? 'date-chip--is-state-2'
          : '',
      ]"
    >
      <button
        type="button"
        class="date-chip__button"
        :class="{ 'date-chip--is-compact': compact }"
        @click="toggle"
      >
        <CalendarIcon :size="compact ? 11 : 12" class="date-chip__part-3" />
        <span class="date-chip__part-4">{{ label }}</span>
      </button>
      <button
        v-if="props.value"
        type="button"
        class="date-chip__"
        :class="{ 'date-chip--is-compact-2': compact }"
        aria-label="Очистить дату"
        @click.stop="clear"
      >
        <X :size="10" />
      </button>
    </div>

    <div
      v-if="open"
      class="date-chip__part-6"
    >
      <Calendar :value="props.value" :today="todayIso" @pick="pick" />
    </div>
  </div>
</template>
