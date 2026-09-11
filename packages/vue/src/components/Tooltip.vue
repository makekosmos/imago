<script setup lang="ts">
import { shallowRef, useId } from "vue";

type TooltipPlacement = "top-start" | "top" | "top-end";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    text: string;
    placement?: TooltipPlacement;
    focusable?: boolean;
  }>(),
  {
    placement: "top",
    focusable: false,
  },
);

const tooltipId = useId();
const visible = shallowRef(false);
const left = shallowRef(0);
const top = shallowRef(0);

function show(event: Event): void {
  if (!(event.currentTarget instanceof HTMLElement)) return;
  const bounds = event.currentTarget.getBoundingClientRect();
  left.value =
    props.placement === "top-start"
      ? bounds.left
      : props.placement === "top-end"
        ? bounds.right
        : bounds.left + bounds.width / 2;
  top.value = bounds.top - 7;
  visible.value = true;
}

function hide(): void {
  visible.value = false;
}
</script>

<template>
  <span
    v-bind="$attrs"
    class="kosmos-tooltip"
    :class="`kosmos-tooltip--${placement}`"
    :tabindex="focusable ? 0 : undefined"
    :aria-label="focusable ? text : undefined"
    :aria-describedby="tooltipId"
    @pointerenter="show"
    @pointerleave="hide"
    @focus="show"
    @blur="hide"
  >
    <slot />
  </span>
  <Teleport to="body">
    <span
      v-if="visible"
      :id="tooltipId"
      class="kosmos-tooltip__content"
      :class="`kosmos-tooltip__content--${placement}`"
      :style="{ left: `${left}px`, top: `${top}px` }"
      role="tooltip"
    >
      {{ text }}
    </span>
  </Teleport>
</template>
