<script setup lang="ts">
import { buttonVariants, type ButtonProps } from '@core/button';


const props = withDefaults(defineProps<ButtonProps>(), {
  loading: false,
  disabled: false,
  type: 'button'
})

</script>

<template>
  <button
    :type="props.type"
    :disabled="props.disabled || props.loading"
    :class="buttonVariants({
      variant: props.variant,
      size: props.size
    })"
    :data-loading="props.loading"
  >
      <span v-if="$slots.icon">
          <slot name="icon"/>
      </span>

      <slot />
  </button>
</template>
