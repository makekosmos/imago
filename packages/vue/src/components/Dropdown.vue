<script setup lang="ts" generic="T extends string | number">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { ChevronDown } from "@lucide/vue";

/**
 * Универсальный Dropdown (shadcn-стиль): кастомный триггер + popover с
 * опциями, teleport в body. v-model по значению (`update:modelValue`).
 * Закрывается по клику снаружи, Escape, выбору.
 */

interface Option<V> {
  value: V;
  label: string;
  /** Визуальный разделитель между группами. Не выбирается и не фильтруется. */
  kind?: "option" | "separator";
  /** Опциональная подсказка под лейблом. */
  description?: string;
  /** Disable конкретной опции. */
  disabled?: boolean;
  /** Monochrome mask icon for the option. */
  iconSrc?: string;
  /** Color for iconSrc/dot. */
  color?: string;
}

interface Props<V> {
  modelValue: V | null;
  options: Option<V>[];
  placeholder?: string;
  /** Если true — pop-up равен ширине триггера. По умолчанию true. */
  matchTriggerWidth?: boolean;
  /** Disable весь триггер. */
  disabled?: boolean;
  /** Показывать ли search field в popup'е. По умолчанию — auto: search
   * появляется когда опций ≥ 6 (для коротких списков он избыточен). */
  searchable?: boolean | "auto";
  /** Placeholder для search input'а. */
  searchPlaceholder?: string;
  /** Максимальная высота popup в px. По умолчанию 200. */
  maxHeightPx?: number;
  /** Выравнивание popup относительно trigger'а. По умолчанию — end. */
  panelAlign?: "start" | "end";
  /** Показывать chevron в trigger'е. */
  showChevron?: boolean;
  /** Создать виртуальную опцию из search query, если её нет в списке. */
  createOption?: (query: string) => Option<V> | null;
}

const props = withDefaults(defineProps<Props<T>>(), {
  placeholder: "Выбрать…",
  matchTriggerWidth: true,
  disabled: false,
  searchable: "auto",
  searchPlaceholder: "Поиск…",
  maxHeightPx: 200,
  panelAlign: "end",
  showChevron: true,
});

const emit = defineEmits<{
  "update:modelValue": [v: T];
}>();

const open = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const panelRef = ref<HTMLElement | null>(null);
const searchInputRef = ref<HTMLInputElement | null>(null);
const optionsRef = ref<HTMLElement | null>(null);
const searchQuery = ref("");
const panelPosition = ref<{ top: number; left: number; width: number }>({
  top: 0,
  left: 0,
  width: 0,
});
const highlightIdx = ref(0);

const isSearchable = computed(() => {
  if (props.searchable === "auto") return props.options.length >= 6;
  return !!props.searchable;
});

const filteredOptions = computed<Option<T>[]>(() => {
  const rawQuery = searchQuery.value.trim();
  const q = rawQuery.toLowerCase();
  if (!q) return props.options;

  const matches = props.options.filter((o) => {
    if (o.kind === "separator") return false;
    return o.label.toLowerCase().includes(q) || Boolean(o.description?.toLowerCase().includes(q));
  });
  const exactMatch = matches.some(
    (o) => o.label.toLowerCase() === q || String(o.value) === rawQuery,
  );
  const created = exactMatch ? null : props.createOption?.(rawQuery);

  return created ? [created, ...matches] : matches;
});

const selectedOption = computed<Option<T> | null>(() => {
  if (props.modelValue === null || props.modelValue === undefined) return null;
  return props.options.find((o) => o.value === props.modelValue) ?? null;
});

const displayLabel = computed(() =>
  selectedOption.value ? selectedOption.value.label : props.placeholder,
);

const panelMaxHeight = computed(() => Math.max(80, Math.min(420, props.maxHeightPx)));
const hasScrollableOptions = ref(false);

function isSelectableOption(opt: Option<T> | undefined): opt is Option<T> {
  return Boolean(opt && !opt.disabled && opt.kind !== "separator");
}

function reposition() {
  const trigger = triggerRef.value;
  const panel = panelRef.value;
  if (!trigger || !panel) return;
  const rect = trigger.getBoundingClientRect();
  const panelH = panel.offsetHeight;
  const panelW = panel.offsetWidth;
  const margin = 4;
  const edgePad = 8;

  // Vertical: выбираем сторону с большим пространством.
  const spaceBelow = window.innerHeight - rect.bottom - margin;
  const spaceAbove = rect.top - margin;
  let top: number;
  if (spaceBelow >= spaceAbove) {
    top = rect.bottom + margin;
  } else {
    top = Math.max(rect.top - panelH - margin, edgePad);
  }

  // Horizontal: по умолчанию anchor по ПРАВОМУ краю trigger'а (panel
  // расходится влево). Для inline/property fields можно выбрать start,
  // чтобы popup начинался от левого края trigger'а.
  let left = props.panelAlign === "start" ? rect.left : rect.right - panelW;
  if (props.panelAlign === "end" && left < edgePad) {
    left = rect.left;
  }
  if (props.panelAlign === "start" && left + panelW > window.innerWidth - edgePad) {
    left = rect.right - panelW;
  }
  // Final clamp в обе стороны viewport.
  left = Math.max(edgePad, Math.min(left, window.innerWidth - panelW - edgePad));

  panelPosition.value = {
    top,
    left,
    width: rect.width,
  };
}

function scrollSelectedOptionIntoView() {
  panelRef.value
    ?.querySelector<HTMLElement>(".kosmos-dd__option--selected")
    ?.scrollIntoView({ block: "nearest" });
}

function scrollHighlightedOptionIntoView() {
  nextTick(() => {
    panelRef.value
      ?.querySelector<HTMLElement>(".kosmos-dd__option--highlighted")
      ?.scrollIntoView({ block: "nearest" });
    updateOptionsOverflow();
  });
}

function updateOptionsOverflow() {
  const el = optionsRef.value;
  hasScrollableOptions.value = Boolean(el && el.scrollHeight > el.clientHeight + 1);
}

function toggle() {
  if (props.disabled) return;
  open.value = !open.value;
  if (open.value) {
    searchQuery.value = "";
    // Подсветим текущий выбранный (или первый) элемент.
    const idx = props.options.findIndex((o) => o.value === props.modelValue);
    highlightIdx.value = idx >= 0 ? idx : props.options.findIndex(isSelectableOption);
    nextTick(() => {
      reposition();
      scrollSelectedOptionIntoView();
      updateOptionsOverflow();
      // Auto-focus в search field (если есть) — UX как в macOS dropdown.
      if (isSearchable.value) searchInputRef.value?.focus();
    });
  }
}

function pick(opt: Option<T>) {
  if (!isSelectableOption(opt)) return;
  emit("update:modelValue", opt.value);
  open.value = false;
}

function onKey(e: KeyboardEvent) {
  if (!open.value) return;
  if (e.key === "Escape") {
    e.preventDefault();
    open.value = false;
    return;
  }
  const opts = filteredOptions.value;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    for (let i = highlightIdx.value + 1; i < opts.length; i++) {
      if (isSelectableOption(opts[i])) {
        highlightIdx.value = i;
        scrollHighlightedOptionIntoView();
        return;
      }
    }
    return;
  }
  if (e.key === "ArrowUp") {
    e.preventDefault();
    for (let i = highlightIdx.value - 1; i >= 0; i--) {
      if (isSelectableOption(opts[i])) {
        highlightIdx.value = i;
        scrollHighlightedOptionIntoView();
        return;
      }
    }
    return;
  }
  if (e.key === "Enter") {
    e.preventDefault();
    const opt = opts[highlightIdx.value];
    if (isSelectableOption(opt)) pick(opt);
    return;
  }
}

// При фильтрации сбрасываем highlight на первую видимую опцию.
watch(searchQuery, () => {
  if (filteredOptions.value.length === 0) {
    highlightIdx.value = -1;
  } else {
    highlightIdx.value = filteredOptions.value.findIndex(isSelectableOption);
  }
  scrollHighlightedOptionIntoView();
  nextTick(updateOptionsOverflow);
});

function onDocPointerDown(e: PointerEvent) {
  if (!open.value) return;
  const t = e.target;
  if (t instanceof Node) {
    if (triggerRef.value?.contains(t)) return;
    if (panelRef.value?.contains(t)) return;
  }
  open.value = false;
}

function onWindowResize() {
  if (!open.value) return;
  reposition();
  nextTick(updateOptionsOverflow);
}

watch(open, (isOpen) => {
  if (isOpen) {
    document.addEventListener("pointerdown", onDocPointerDown);
    document.addEventListener("keydown", onKey);
    window.addEventListener("dropdown__part-19", onWindowResize);
    window.addEventListener("scroll", onWindowResize, true);
  } else {
    document.removeEventListener("pointerdown", onDocPointerDown);
    document.removeEventListener("keydown", onKey);
    window.removeEventListener("dropdown__part-23", onWindowResize);
    window.removeEventListener("scroll", onWindowResize, true);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", onDocPointerDown);
  document.removeEventListener("keydown", onKey);
  window.removeEventListener("dropdown__part-27", onWindowResize);
  window.removeEventListener("scroll", onWindowResize, true);
});
</script>

<template>
  <div class="dropdown">
    <button
      ref="triggerRef"
      type="button"
      class="dropdown__trigger-ref"
      :class="{
        'dropdown--is-open': open,
        'dropdown--is-selected-option': !selectedOption,
        'dropdown--is-disabled': disabled,
      }"
      :disabled="disabled"
      :aria-haspopup="'listbox'"
      :aria-expanded="open"
      :style="{ fontSize: 'var(--kosmos-settings-font-size-base, 12px)' }"
      @click="toggle"
    >
      <!-- Слот для leading-иконки в trigger'е (показывает иконку текущего
           выбранного варианта). Если не передан — без иконки. -->
      <slot name="trigger-leading" :option="selectedOption" />
      <span class="dropdown__part-3">
        {{ displayLabel }}
      </span>
      <ChevronDown
        v-if="showChevron"
        class="dropdown__part-4"
        :class="{ 'dropdown--is-open-2': open }"
        :size="12"
        :stroke-width="2"
      />
    </button>

    <Teleport to="body">
      <transition name="kosmos-dd">
        <div
          v-if="open"
          ref="panelRef"
          class="kosmos-dd__panel dropdown__panel-ref"
          role="listbox"
          :style="{
            top: panelPosition.top + 'px',
            left: panelPosition.left + 'px',
            width: matchTriggerWidth ? panelPosition.width + 'px' : undefined,
            maxHeight: `min(${panelMaxHeight}px, calc(100vh - 32px))`,
          }"
        >
          <div v-if="isSearchable" class="kosmos-dd__search">
            <input
              ref="searchInputRef"
              v-model="searchQuery"
              type="text"
              class="dropdown__search-input-ref"
              :placeholder="searchPlaceholder"
              spellcheck="false"
              autocomplete="off"
              :style="{
                fontSize: 'var(--kosmos-settings-font-size-base, 12px)',
              }"
            />
          </div>
          <div
            ref="optionsRef"
            class="kosmos-dd__options kosmos-scroll dropdown__options-ref"
            :class="{ 'kosmos-dd__options--scrollable': hasScrollableOptions }"
          >
            <template v-for="(opt, i) in filteredOptions" :key="String(opt.value)">
              <div
                v-if="opt.kind === 'separator'"
                class="kosmos-dd__separator dropdown__part-9"
                role="separator"
                aria-hidden="true"
              ></div>
              <button
                v-else
                type="button"
                class="kosmos-dd__option dropdown__button"
                :class="{
                  'kosmos-dd__option--selected': opt.value === modelValue,
                  'kosmos-dd__option--highlighted': i === highlightIdx,
                  'dropdown--is-disabled-2': opt.disabled,
                }"
                role="option"
                :aria-selected="opt.value === modelValue"
                :disabled="opt.disabled"
                :style="{
                  fontSize: 'var(--kosmos-settings-font-size-base, 12px)',
                }"
                @mouseenter="isSelectableOption(opt) && (highlightIdx = i)"
                @click="pick(opt)"
              >
                <slot name="option-leading" :option="opt">
                  <span
                    v-if="opt.iconSrc || opt.color"
                    class="kosmos-dd__option-swatch"
                    :style="{
                      '--kosmos-dd-option-color': opt.color ?? 'var(--text-secondary)',
                    }"
                    aria-hidden="true"
                  >
                    <span
                      v-if="opt.iconSrc && opt.color"
                      class="kosmos-dd__option-icon"
                      :style="{
                        '--kosmos-dd-option-icon-src': `url(${opt.iconSrc})`,
                      }"
                    ></span>
                    <img
                      v-else-if="opt.iconSrc"
                      class="kosmos-dd__option-img"
                      :src="opt.iconSrc"
                      alt=""
                    />
                    <span v-else class="kosmos-dd__option-dot"></span>
                  </span>
                </slot>
                <span class="dropdown__part-15">
                  {{ opt.label }}
                </span>
              </button>
            </template>
            <div
              v-if="filteredOptions.length === 0"
              class="dropdown__part-16"
            >
              Ничего не найдено
            </div>
          </div>
        </div>
      </transition>
    </Teleport>
  </div>
</template>
