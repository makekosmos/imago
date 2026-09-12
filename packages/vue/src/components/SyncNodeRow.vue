<script setup lang="ts">
import { computed } from "vue";
import { Laptop, Monitor, Smartphone } from "@lucide/vue";
import Button from "./Button.vue";
import StatusDot from "./StatusDot.vue";

type SyncNodeDeviceKind = "desktop" | "laptop" | "phone" | "unknown";
type SyncNodeStatus = "online" | "offline" | "connecting";

const props = withDefaults(
  defineProps<{
    deviceKind: SyncNodeDeviceKind;
    name: string;
    lastSeenLabel: string;
    status: SyncNodeStatus;
    statusLabel?: string;
    disconnectLabel?: string;
    disabled?: boolean;
    disconnecting?: boolean;
  }>(),
  {
    statusLabel: undefined,
    disconnectLabel: "Отключить",
    disabled: false,
    disconnecting: false,
  },
);

const emit = defineEmits<{ disconnect: [] }>();

const icon = computed(() => {
  switch (props.deviceKind) {
    case "phone":
      return Smartphone;
    case "laptop":
      return Laptop;
    default:
      return Monitor;
  }
});

const tone = computed(() =>
  props.status === "online" ? "success" : props.status === "connecting" ? "warning" : "danger",
);
</script>

<template>
  <div
    class="group sync-node-row"
  >
    <div
      class="sync-node-row__part-2"
    >
      <component :is="icon" :size="18" :stroke-width="2" />
    </div>

    <div class="sync-node-row__part-3">
      <div
        class="sync-node-row__part-4"
      >
        {{ name }}
      </div>
      <div
        class="sync-node-row__part-5"
      >
        {{ lastSeenLabel }}
      </div>
    </div>

    <div class="sync-node-row__part-6">
      <Button
        size="sm"
        variant="danger"
        :loading="disconnecting"
        :disabled="disabled || disconnecting"
        class="sync-node-row__part-7"
        @click="emit('disconnect')"
      >
        {{ disconnectLabel }}
      </Button>
      <StatusDot :tone="tone" :label="statusLabel ?? status" />
    </div>
  </div>
</template>
