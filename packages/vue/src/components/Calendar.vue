<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import Dropdown from "./Dropdown.vue";

interface Props {
  /** Выбранная дата в виде ISO `YYYY-MM-DD` (без времени). null = ничего не выбрано. */
  value: string | null;
  /** Сегодняшняя дата (для подсветки). По умолчанию — текущий локальный день. */
  today?: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "update:value": [iso: string];
  pick: [iso: string];
}>();

const RU_WEEKDAYS_SHORT = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"] as const;
const RU_MONTHS_LOWER = [
  "январь",
  "февраль",
  "март",
  "апрель",
  "май",
  "июнь",
  "июль",
  "август",
  "сентябрь",
  "октябрь",
  "ноябрь",
  "декабрь",
] as const;

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

function formatIsoYear(year: number): string {
  if (year < 0) return `-${String(Math.abs(year)).padStart(6, "0")}`;
  return String(year).padStart(4, "0");
}

function localDate(year: number, month: number, day: number): Date {
  const d = new Date(0, month, day);
  d.setFullYear(year);
  return d;
}

function toIso(d: Date): string {
  return `${formatIsoYear(d.getFullYear())}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function parseIso(iso: string | null): Date | null {
  if (!iso) return null;
  const m = /^([+-]?\d{4,6})-(\d{2})-(\d{2})/.exec(iso);
  if (!m) return null;
  return localDate(Number(m[1]), Number(m[2]) - 1, Number(m[3]));
}

const todayDate = computed(() => {
  if (props.today) return parseIso(props.today) ?? new Date();
  return new Date();
});

const selectedDate = computed(() => parseIso(props.value));
const viewMonth = ref<Date>(
  localDate(
    (selectedDate.value ?? todayDate.value).getFullYear(),
    (selectedDate.value ?? todayDate.value).getMonth(),
    1,
  ),
);

watch(
  () => props.value,
  (iso) => {
    const d = parseIso(iso);
    if (d) viewMonth.value = localDate(d.getFullYear(), d.getMonth(), 1);
  },
);

const yearOptions = computed(() => {
  const currentYear = todayDate.value.getFullYear();
  const years: { value: number; label: string }[] = [];
  for (let year = currentYear; year > currentYear - 100; year--) {
    years.push({ value: year, label: String(year) });
  }

  if (!years.some((option) => option.value === viewYear.value)) {
    years.unshift({ value: viewYear.value, label: String(viewYear.value) });
  }

  return years;
});

interface DayCell {
  date: Date;
  iso: string;
  day: number;
  isToday: boolean;
  isSelected: boolean;
  isOutsideMonth: boolean;
}

const cells = computed<DayCell[]>(() => {
  const out: DayCell[] = [];
  const todayIso = toIso(todayDate.value);
  const selectedIso = selectedDate.value ? toIso(selectedDate.value) : null;
  const first = new Date(viewMonth.value.getFullYear(), viewMonth.value.getMonth(), 1);
  const jsDow = first.getDay();
  const mondayOffset = (jsDow + 6) % 7;
  const gridStart = new Date(first);
  gridStart.setDate(first.getDate() - mondayOffset);

  for (let i = 0; i < 42; i++) {
    const d = new Date(gridStart);
    d.setDate(gridStart.getDate() + i);
    const iso = toIso(d);
    out.push({
      date: d,
      iso,
      day: d.getDate(),
      isToday: iso === todayIso,
      isSelected: iso === selectedIso,
      isOutsideMonth: d.getMonth() !== viewMonth.value.getMonth(),
    });
  }
  return out;
});

const viewYear = computed(() => viewMonth.value.getFullYear());
const viewMonthIndex = computed(() => viewMonth.value.getMonth());

function shiftMonth(delta: number) {
  viewMonth.value = localDate(viewMonth.value.getFullYear(), viewMonth.value.getMonth() + delta, 1);
}

function setYear(rawYear: string | number) {
  const year = Number(rawYear);
  if (!Number.isInteger(year)) return;
  viewMonth.value = localDate(year, viewMonth.value.getMonth(), 1);
}

function createYearOption(query: string) {
  if (!/^-?\d{1,6}$/.test(query)) return null;
  const year = Number(query);
  if (!Number.isInteger(year)) return null;
  return { value: year, label: String(year) };
}

function pickCell(c: DayCell) {
  emit("update:value", c.iso);
  emit("pick", c.iso);
}
</script>

<template>
  <div
    class="calendar"
  >
    <header class="calendar__header">
      <div class="calendar__part-3">
        <Dropdown
          class="calendar-year-dropdown"
          :model-value="viewYear"
          :options="yearOptions"
          :match-trigger-width="false"
          panel-align="start"
          :show-chevron="false"
          :max-height-px="260"
          :create-option="createYearOption"
          @update:model-value="setYear"
        />
        <span>, {{ RU_MONTHS_LOWER[viewMonthIndex] }}</span>
      </div>

      <div class="calendar__part-5">
        <button
          type="button"
          class="calendar__"
          aria-label="Предыдущий месяц"
          @click="shiftMonth(-1)"
        >
          <ChevronLeft :size="14" :stroke-width="1.8" />
        </button>
        <button
          type="button"
          class="calendar__"
          aria-label="Следующий месяц"
          @click="shiftMonth(1)"
        >
          <ChevronRight :size="14" :stroke-width="1.8" />
        </button>
      </div>
    </header>

    <div class="calendar__part-8">
      <div
        v-for="weekday in RU_WEEKDAYS_SHORT"
        :key="weekday"
        class="calendar__part-9"
      >
        {{ weekday }}
      </div>

      <button
        v-for="c in cells"
        :key="c.iso"
        type="button"
        class="calendar-day calendar__button"
        :class="{
          'calendar--is-is-outside-month': c.isOutsideMonth,
          'calendar--is-is-selected':
            c.isToday && !c.isSelected,
          'calendar-day--selected calendar--is-is-selected-2':
            c.isSelected,
        }"
        @click="pickCell(c)"
      >
        {{ c.day }}
      </button>
    </div>
  </div>
</template>
