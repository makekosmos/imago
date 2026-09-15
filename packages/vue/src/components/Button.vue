<script setup lang="ts">
import { buttonVariants, type ButtonSharedProps } from '@core/components/button';


const props = withDefaults(defineProps<ButtonSharedProps>(), {
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
