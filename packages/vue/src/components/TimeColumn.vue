<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";

interface Props {
  /** Текущее выбранное значение (число). */
  value: number;
  /** Минимум (inclusive). */
  min?: number;
  /** Максимум (inclusive). */
  max: number;
  /** Шаг (по умолчанию 1). */
  step?: number;
  /** Aria-label для скриниридера. */
  label?: string;
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  step: 1,
  label: undefined,
});

const emit = defineEmits<{
  "update:value": [v: number];
}>();

const items = computed<number[]>(() => {
  const out: number[] = [];
  for (let v = props.min; v <= props.max; v += props.step) out.push(v);
  return out;
});

const root = ref<HTMLElement | null>(null);

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

function scrollSelectedIntoView(behavior: ScrollBehavior = "smooth") {
  if (!root.value) return;
  const el = root.value.querySelector<HTMLElement>(`[data-value="${props.value}"]`);
  if (el) el.scrollIntoView({ block: "center", behavior });
}

onMounted(() => {
  nextTick(() => scrollSelectedIntoView("instant"));
});

watch(
  () => props.value,
  () => scrollSelectedIntoView("smooth"),
);
</script>

<template>
  <div
    ref="root"
    class="kosmos-timecol time-column__root"
    :aria-label="props.label"
    role="listbox"
  >
    <button
      v-for="v in items"
      :key="v"
      type="button"
      role="option"
      class="time-column__button"
      :class="{
        'time-column--is-value':
          v === props.value,
      }"
      :aria-selected="v === props.value"
      :data-value="v"
      @click="emit('update:value', v)"
    >
      {{ pad(v) }}
    </button>
  </div>
</template>
