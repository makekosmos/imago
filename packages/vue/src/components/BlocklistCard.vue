<script setup lang="ts">
// BlocklistCard — карточка блок-листа в Settings → Focus.
// Сверху preview содержимого (домены) с gradient fade, снизу footer
// с иконкой и заголовком + опциональным subtitle (количество доменов).
//
// Slot `default` → override preview (например custom markdown с @mentions).
// Slot `actions` → дополнительные кнопки в footer рядом с title.

import { computed } from "vue";

interface Props {
  name: string;
  domains: string[];
  icon?: string;
  preset?: boolean;
  active?: boolean;
  count?: number;
}

const props = withDefaults(defineProps<Props>(), {
  icon: "",
  preset: false,
  active: false,
  count: undefined,
});

const emit = defineEmits<{
  click: [];
  delete: [];
}>();

const effectiveCount = computed(() => props.count ?? props.domains.length);

const subtitle = computed(() => {
  const n = effectiveCount.value;
  if (n === 0) return "пусто";
  // Russian pluralization для "домен / домена / доменов"
  const mod10 = n % 10;
  const mod100 = n % 100;
  if (mod10 === 1 && mod100 !== 11) return `${n} домен`;
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) return `${n} домена`;
  return `${n} доменов`;
});

const previewLines = computed(() => props.domains.slice(0, 12));

function onClick() {
  emit("click");
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    emit("click");
  }
}
</script>

<template>
  <div
    class="blocklist-card"
    :class="{
      'blocklist-card--is-active':
        active,
    }"
    role="button"
    tabindex="0"
    @click="onClick"
    @keydown="onKey"
  >
    <div
      class="blocklist-card__part-2"
      aria-hidden="true"
    >
      <slot>
        <div
          v-for="(domain, i) in previewLines"
          :key="i"
          class="blocklist-card__part-3"
        >
          {{ domain }}
        </div>
      </slot>

      <span
        v-if="previewLines.length === 0"
        class="blocklist-card__part-4"
      >
        —
      </span>

      <span
        v-if="preset"
        class="blocklist-card__part-5"
      >
        preset
      </span>

      <span
        v-if="active"
        class="blocklist-card__part-6"
        aria-hidden="true"
      />
    </div>

    <div
      class="blocklist-card__part-7"
    >
      <span v-if="icon" class="blocklist-card__part-8">{{ icon }}</span>
      <div class="blocklist-card__part-9">
        <div
          class="blocklist-card__part-10"
        >
          {{ name }}
        </div>
        <div
          class="blocklist-card__part-11"
        >
          {{ subtitle }}
        </div>
      </div>
      <div class="blocklist-card__part-12">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>
