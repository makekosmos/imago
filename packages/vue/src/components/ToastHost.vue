<script setup lang="ts">
import { inject } from "vue";
import { ToastKey } from "../composables/useToast";
import Toast from "./Toast.vue";

// ToastHost — визуальный renderer. provide делает родитель через
// provideToastHost() в setup'е; этот компонент только inject'ит и рисует.
const state = inject(ToastKey, null);
</script>

<template>
  <Teleport v-if="state" to="body">
    <div
      class="toast-host"
      aria-live="polite"
    >
      <TransitionGroup name="kosmos-toast">
        <Toast
          v-for="item in state.items.value"
          :key="item.id"
          :message="item.message"
          :title="item.title"
          :description="item.description"
          :tone="item.tone"
          :loading="item.loading"
          :closable="item.closable"
          @dismiss="state.api.dismiss(item.id)"
        />
      </TransitionGroup>
    </div>
  </Teleport>
</template>
