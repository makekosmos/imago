<script setup lang="ts">
import { ref } from "vue";

const modalOpen = ref(false);
</script>

<template>
  <main class="preview">
    <h1>Vue preview</h1>

    <section>
      <h2>Buttons</h2>
      <div class="rule" />
      <div class="row">
        <button class="primary">Primary</button>
        <button>Ghost</button>
        <button class="danger">Danger</button>
      </div>
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
