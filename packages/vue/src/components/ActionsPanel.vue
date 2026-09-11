<script setup lang="ts">
import KbdKey from "./KbdKey.vue";

defineProps<{
  isFavorite: boolean;
  isHidden: boolean;
  canHide: boolean;
}>();

defineEmits<{
  open: [];
  toggleFavorite: [];
  toggleHide: [];
}>();
</script>

<template>
  <div class="actions-panel">
    <ul class="actions-panel__list">
      <li class="actions-panel__item" @click="$emit('open')">
        <span class="actions-panel__label">Открыть</span>
        <span class="actions-panel__keys"><KbdKey>↵</KbdKey></span>
      </li>
      <li class="actions-panel__item" @click="$emit('toggleFavorite')">
        <span class="actions-panel__label">{{
          isFavorite ? "Убрать из избранного" : "Добавить в избранное"
        }}</span>
        <span class="actions-panel__keys">
          <KbdKey>Ctrl</KbdKey><KbdKey>Shift</KbdKey><KbdKey>F</KbdKey>
        </span>
      </li>
      <li v-if="canHide" class="actions-panel__item" @click="$emit('toggleHide')">
        <span class="actions-panel__label">{{
          isHidden ? "Показать в списке" : "Скрыть из списка"
        }}</span>
        <span class="actions-panel__keys"> <KbdKey>Ctrl</KbdKey><KbdKey>H</KbdKey> </span>
      </li>
    </ul>
  </div>
</template>
