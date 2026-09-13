<script setup lang="ts">
import { CalendarDays, Inbox } from "@lucide/vue";
import { Button, Sidebar } from "../../../packages/vue/src";
import { computed, ref } from "vue";

const modalOpen = ref(false);
const selectedSidebarItem = ref("inbox");
const sidebarItems = computed(() => [
  {
    id: "inbox",
    icon: Inbox,
    label: "Inbox",
    active: selectedSidebarItem.value === "inbox",
    onClick: () => { selectedSidebarItem.value = "inbox"; },
  },
  {
    id: "today",
    icon: CalendarDays,
    label: "Today",
    active: selectedSidebarItem.value === "today",
    onClick: () => { selectedSidebarItem.value = "today"; },
  },
]);
</script>

<template>
  <main class="preview">
    <h1>Vue preview</h1>

    <section>
      <h2>Buttons</h2>
      <div class="rule" />
      <div class="row">
        <Button>Primary</Button>
        <Button variant="ghost">Ghost</Button>
        <Button variant="danger">Danger</Button>
      </div>
    </section>

    <section class="sidebar-preview">
      <h2>Sidebar buttons</h2>
      <div class="rule" />
      <Sidebar
        :primary-items="sidebarItems"
        :show-toggle="false"
        :drag-region="false"
        :reserve-top-inset="false"
        :default-width="220"
        :min-width="220"
        :max-width="220"
      />
    </section>

    <section>
      <h2>Inputs</h2>
      <div class="rule" />
      <input type="text" placeholder="text input" aria-label="Text input" />
    </section>

    <section>
      <h2>Modal</h2>
      <div class="rule" />
      <button class="primary" @click="modalOpen = true">Open modal</button>
    </section>

    <section>
      <h2>Dropdown</h2>
      <div class="rule" />
      <select aria-label="Select">
        <option>Select</option>
        <option>Option one</option>
        <option>Option two</option>
      </select>
    </section>

    <div v-if="modalOpen" class="backdrop" @click.self="modalOpen = false">
      <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <h2 id="modal-title">Modal</h2>
        <button @click="modalOpen = false">Close</button>
      </div>
    </div>
  </main>
</template>
