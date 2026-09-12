<script setup lang="ts">
// Button — общий primary/ghost/danger primitive поверх @kosmos/visuals tokens.
// Заменяет ad-hoc `.btn` / `.btn.ghost` CSS, тиражирующиеся в SettingsView и
// прочих местах. v-slot для текста + опциональный leading icon через
// именованный slot `icon`.

withDefaults(
  defineProps<{
    /** Внешний вид. */
    variant?: "primary" | "ghost" | "surface" | "success" | "danger";
    /** Размер: md (default, 40px h) или sm (28px h). */
    size?: "md" | "sm";
    /** Кнопка занимает всю ширину контейнера. */
    block?: boolean;
    /** В состоянии загрузки текст приглушён, события не идут. */
    loading?: boolean;
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  {
    variant: "primary",
    size: "md",
    block: false,
    loading: false,
    disabled: false,
    type: "button",
  },
);
</script>

<template>
  <button
    :type="type"
    :disabled="disabled || loading"
    :class="[
      'button',
      size === 'sm'
        ? 'button-sm'
        : 'button-md',
      block ? 'button-block' : '',
      variant === 'primary'
        ? 'button-primary'
        : '',
      variant === 'ghost'
        ? 'button-ghost'
        : '',
      variant === 'surface'
      ? 'button-surface'
        : '',
      variant === 'success'
        ? 'button-success'
        : '',
      variant === 'danger'
        ? 'button-danger'
        : '',
    ]"
  >
    <span v-if="$slots.icon" class="button"><slot name="icon" /></span>
    <span :class="loading ? 'button-loading' : ''"><slot /></span>
  </button>
</template>
