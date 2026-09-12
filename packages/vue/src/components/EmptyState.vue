<script setup lang="ts">
// EmptyState — стандартный empty-state для list views, trash, search results.
// Используется в Eden (SearchOverlay nothing), Delphi (QuickSearch empty),
// Arrancador (game list empty).
//
// title — заголовок (например «Нет заметок»)
// description — поясняющий текст (опционально)
// Slot `action` — кнопка действия (например «Создать первую заметку»)
// Slot `icon` — кастомный icon (по умолчанию empty)

interface Props {
  title: string;
  description?: string;
  /** Уменьшенный вариант для компактных контейнеров (popover, sidebar). */
  compact?: boolean;
}

withDefaults(defineProps<Props>(), {
  compact: false,
});
</script>

<template>
  <div
    class="empty-state"
    :class="compact ? 'empty-state--is-py' : 'empty-state--is-state'"
  >
    <div
      v-if="$slots.icon"
      class="empty-state__part-2"
    >
      <slot name="icon" />
    </div>
    <div class="empty-state__part-3" :class="compact ? 'empty-state--is-rem' : 'empty-state--is-state-2'">
      {{ title }}
    </div>
    <div v-if="description" class="empty-state__part-4">
      {{ description }}
    </div>
    <div v-if="$slots.action" class="empty-state__part-5">
      <slot name="action" />
    </div>
  </div>
</template>
