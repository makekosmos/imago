<script setup lang="ts">
import { ChevronLeft, ChevronRight } from "@lucide/vue";
import TitlebarButton from "./TitlebarButton.vue";

interface Props {
  backDisabled?: boolean;
  forwardDisabled?: boolean;
  backTitle?: string;
  forwardTitle?: string;
}

const props = withDefaults(defineProps<Props>(), {
  backDisabled: false,
  forwardDisabled: false,
  backTitle: "Назад",
  forwardTitle: "Вперёд",
});

const emit = defineEmits<{
  back: [];
  forward: [];
}>();

function handleBack() {
  if (props.backDisabled) return;
  emit("back");
}

function handleForward() {
  if (props.forwardDisabled) return;
  emit("forward");
}
</script>

<template>
  <div class="kosmos-titlebar-history-controls titlebar-history-controls">
    <TitlebarButton
      :disabled="backDisabled"
      :title="backTitle"
      :aria-label="backTitle"
      data-testid="titlebar-history-back"
      @click="handleBack"
    >
      <ChevronLeft :size="16" />
    </TitlebarButton>

    <TitlebarButton
      :disabled="forwardDisabled"
      :title="forwardTitle"
      :aria-label="forwardTitle"
      data-testid="titlebar-history-forward"
      @click="handleForward"
    >
      <ChevronRight :size="16" />
    </TitlebarButton>
  </div>
</template>
