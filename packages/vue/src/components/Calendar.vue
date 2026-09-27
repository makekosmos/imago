<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import { formatIsoYear, localDate, pad } from "./dates";
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

function toIso(d: Date): string {
  return `${formatIsoYear(d.getFullYear())}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function parseIso(iso: string | null): Date | null {
  if (!iso) return null;
  const m = /^([+-]?\d{4,6})-(\d{2})-(\d{2})/.exec(iso);
  if (!m) return null;
  const year = Number(m[1]);
  const month = Number(m[2]);
  const day = Number(m[3]);
  // Год за пределами диапазона Date (±~275760) даёт Invalid Date — без
  // проверки NaN стекался в viewMonth/cells и вся сетка становилась "NaN".
  const d = localDate(year, month - 1, day);
  if (Number.isNaN(d.getTime())) return null;
  // Переполненные поля (месяц 13, 31 февраля) localDate молча сдвигает на
  // другую реальную дату — календарь подсвечивал бы не то число, что лежит
  // в value. Такое значение malformed → считаем его отсутствующим.
  if (d.getFullYear() !== year || d.getMonth() !== month - 1 || d.getDate() !== day) {
    return null;
  }
  return d;
}

const todayDate = computed(() => {
  if (props.today) return parseIso(props.today) ?? new Date();
  return new Date();
});

const selectedDate = computed(() => parseIso(props.value));

// Якорь месяца — существующая дата внутри него (viewMonth никогда не
// хранит Invalid Date). У нижней границы диапазона (апрель -271821) 1-е
// число не существует — якоримся на последний день месяца. null — если
// месяц целиком вне диапазона Date.
function monthAnchor(year: number, month: number): Date | null {
  const first = localDate(year, month, 1);
  if (!Number.isNaN(first.getTime())) return first;
  const last = localDate(year, month + 1, 0);
  return Number.isNaN(last.getTime()) ? null : last;
}

const viewMonth = ref<Date>(
  monthAnchor(
    (selectedDate.value ?? todayDate.value).getFullYear(),
    (selectedDate.value ?? todayDate.value).getMonth(),
  ) ?? todayDate.value,
);

watch(
  () => props.value,
  (iso) => {
    const d = parseIso(iso);
    const anchor = d && monthAnchor(d.getFullYear(), d.getMonth());
    if (anchor) viewMonth.value = anchor;
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
  /** "" у ячеек за пределами диапазона Date — такие дни не существуют. */
  iso: string;
  /** null для несуществующих дней — ячейка рисуется пустой и не кликается. */
  day: number | null;
  isToday: boolean;
  isSelected: boolean;
  isOutsideMonth: boolean;
}

const cells = computed<DayCell[]>(() => {
  const out: DayCell[] = [];
  const todayIso = toIso(todayDate.value);
  const selectedIso = selectedDate.value ? toIso(selectedDate.value) : null;
  const year = viewMonth.value.getFullYear();
  const month = viewMonth.value.getMonth();
  // День недели 1-го числа — от якоря: сама дата 1-го числа может не
  // существовать (апрель -271821). getDay(): вс=0 → сдвигаем к пн=0.
  const anchorDow = (viewMonth.value.getDay() + 6) % 7;
  const mondayOffset = (((anchorDow - (viewMonth.value.getDate() - 1)) % 7) + 7) % 7;

  for (let i = 0; i < 42; i++) {
    // Каждая ячейка — отдельный localDate: переполнение дня решается внутри
    // целевого месяца, а дни за пределами диапазона Date (последняя неделя
    // сентября 275760, первая — апреля -271821) дают Invalid Date — их
    // рендерим пустыми некликабельными ячейками, а не "NaN".
    const d = localDate(year, month, 1 + i - mondayOffset);
    const valid = !Number.isNaN(d.getTime());
    const iso = valid ? toIso(d) : "";
    out.push({
      date: d,
      iso,
      day: valid ? d.getDate() : null,
      isToday: valid && iso === todayIso,
      isSelected: valid && iso === selectedIso,
      isOutsideMonth: !valid || d.getMonth() !== month,
    });
  }
  return out;
});

const viewYear = computed(() => viewMonth.value.getFullYear());
const viewMonthIndex = computed(() => viewMonth.value.getMonth());

function shiftMonth(delta: number) {
  // У границы диапазона Date соседнего месяца может не быть вовсе — без
  // проверки viewMonth уходил в NaN: сетка рисовала "NaN"-ячейки, а
  // setYear не мог вернуть календарь. Частично валидный месяц (апрель
  // -271821) якорится на последний существующий день.
  const next = monthAnchor(viewMonth.value.getFullYear(), viewMonth.value.getMonth() + delta);
  if (!next) return;
  viewMonth.value = next;
}

function isValidViewDate(year: number, month: number): boolean {
  return monthAnchor(year, month) !== null;
}

function setYear(rawYear: string | number) {
  const year = Number(rawYear);
  if (!Number.isInteger(year)) return;
  const anchor = monthAnchor(year, viewMonth.value.getMonth());
  if (!anchor) return;
  viewMonth.value = anchor;
}

function createYearOption(query: string) {
  if (!/^-?\d{1,6}$/.test(query)) return null;
  const year = Number(query);
  if (!Number.isInteger(year)) return null;
  // 6 цифр позволяют выйти за диапазон Date (|year| > ~275760 → Invalid
  // Date) — такой год нельзя выбрать: сетка превращалась бы в "NaN".
  if (!isValidViewDate(year, 0)) return null;
  return { value: year, label: String(year) };
}

function pickCell(c: DayCell) {
  if (!c.iso) return; // пустая ячейка за пределами диапазона Date
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
        v-for="(c, i) in cells"
        :key="c.iso || `empty-${i}`"
        type="button"
        class="calendar-day calendar__button"
        :disabled="!c.iso"
        :aria-hidden="!c.iso"
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
