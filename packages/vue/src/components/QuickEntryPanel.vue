<script setup lang="ts">
import { shallowRef, watch, nextTick, useTemplateRef, onBeforeUnmount } from "vue";
import { DollarSign, Folder, X } from "@lucide/vue";
import DateChip from "./DateChip.vue";
import type { QuickEntryProject, QuickEntrySavePayload } from "./types";

const props = defineProps<{
  open: boolean;
  projects?: QuickEntryProject[];
  defaultScheduledDate?: string | null;
  defaultProjectId?: string | null;
}>();

const emit = defineEmits<{
  "update:open": [boolean];
  save: [payload: QuickEntrySavePayload];
}>();

const title = shallowRef("");
const notes = shallowRef("");
const scheduledDate = shallowRef<string | null>(null);
const selectedProjectId = shallowRef<string | null>(null);
const billable = shallowRef(false);
const priceInput = shallowRef("");
const showProjectMenu = shallowRef(false);

const titleRef = useTemplateRef<HTMLInputElement>("titleInput");
const projectMenuRef = useTemplateRef<HTMLDivElement>("projectMenu");

const selectedProject = () => props.projects?.find((p) => p.id === selectedProjectId.value);

function reset() {
  title.value = "";
  notes.value = "";
  scheduledDate.value = props.defaultScheduledDate ?? null;
  selectedProjectId.value = props.defaultProjectId ?? null;
  billable.value = Boolean(selectedProject()?.billable);
  priceInput.value = "";
  showProjectMenu.value = false;
}

function close() {
  emit("update:open", false);
  reset();
}

function save() {
  const trimmed = title.value.trim();
  if (!trimmed) {
    close();
    return;
  }
  const parsedPrice = priceInput.value.trim() === "" ? null : Number(priceInput.value);
  emit("save", {
    title: trimmed,
    notes: notes.value || null,
    scheduledDate: scheduledDate.value,
    projectId: selectedProjectId.value,
    billable: billable.value,
    price: Number.isFinite(parsedPrice) ? parsedPrice : null,
  });
  close();
}

function inputValue(event: Event): string {
  // SAFETY: these handlers are attached only to input and textarea elements.
  return (event.target as HTMLInputElement | HTMLTextAreaElement).value;
}

function onTitleKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    save();
  }
  if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}

function onNotesKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}

watch(
  () => props.open,
  (val) => {
    if (val) {
      reset();
      nextTick(() => titleRef.value?.focus());
    }
  },
);

// Outside-click для project menu. Раньше использовался nested watch({ once: true })
// для cleanup'а — он ломался при последовательных open/close (новый handler
// добавлялся раньше чем предыдущий cleanup'ался) и не отписывался на unmount.
// Идиома `watch open ⇒ add/remove` + `onBeforeUnmount → remove` — symmetric и
// безопасна (mirror ContextMenu.vue).
function onProjectMenuOutsideClick(e: MouseEvent) {
  // SAFETY: MouseEvent.target is a DOM node for events dispatched by document.
  if (projectMenuRef.value && e.target instanceof Node && !projectMenuRef.value.contains(e.target)) {
    showProjectMenu.value = false;
  }
}

watch(showProjectMenu, (val) => {
  if (val) {
    document.addEventListener("mousedown", onProjectMenuOutsideClick);
  } else {
    document.removeEventListener("mousedown", onProjectMenuOutsideClick);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener("mousedown", onProjectMenuOutsideClick);
});
</script>

<template>
  <div v-if="open" class="quick-entry-panel">
    <!-- Backdrop -->
    <div
      class="quick-entry-panel__part-2"
      @click="close"
    />

    <!-- Panel -->
    <div
      class="quick-entry-panel__part-3"
    >
      <div class="quick-entry-panel__part-4">
        <input
          ref="titleInput"
          type="text"
          placeholder="Новая задача"
          :value="title"
          class="quick-entry-panel__title-input"
          @input="title = inputValue($event)"
          @keydown="onTitleKeyDown"
        />

        <textarea
          placeholder="Заметки"
          :value="notes"
          :rows="2"
          class="quick-entry-panel__textarea"
          @input="notes = inputValue($event)"
          @keydown="onNotesKeyDown"
        />
      </div>

      <div class="quick-entry-panel__part-7" />

      <div class="quick-entry-panel__part-8">
        <div class="quick-entry-panel__part-9">
          <DateChip
            :value="scheduledDate"
            placeholder="Без даты"
            @update:value="scheduledDate = $event"
          />

          <button
            type="button"
            :class="[
              'quick-entry-panel--is-state',
              billable
                ? 'quick-entry-panel--is-surface'
                : 'quick-entry-panel--is-state-2',
            ]"
            :title="billable ? 'Оплачиваемая задача' : 'Сделать оплачиваемой'"
            @click="billable = !billable"
          >
            <DollarSign :size="16" />
            <span>Оплачиваемая</span>
          </button>

          <label
            v-if="billable"
            class="quick-entry-panel__part-10"
          >
            <input
              type="number"
              inputmode="decimal"
              min="0"
              step="0.01"
              placeholder="Цена"
              :value="priceInput"
              class="quick-entry-panel__input"
              @input="priceInput = inputValue($event)"
            />
          </label>

          <div class="quick-entry-panel__part-12" />

          <div ref="projectMenu" class="quick-entry-panel__project-menu">
            <button
              type="button"
              :class="[
                'quick-entry-panel--is-state',
                selectedProject()
                  ? 'quick-entry-panel--is-surface-2'
                  : 'quick-entry-panel--is-state-2',
              ]"
              @click="showProjectMenu = !showProjectMenu"
            >
              <Folder :size="16" />
              <span>{{ selectedProject()?.title ?? "Входящие" }}</span>
            </button>

            <div
              v-if="showProjectMenu"
              class="quick-entry-panel__part-14"
            >
              <button
                type="button"
                :class="[
                  'quick-entry-panel--is-state-3',
                  selectedProjectId === null ? 'quick-entry-panel--is-state-4' : '',
                ]"
                @click="
                  selectedProjectId = null;
                  billable = false;
                  showProjectMenu = false;
                "
              >
                Входящие
              </button>
              <button
                v-for="project in projects ?? []"
                :key="project.id"
                type="button"
                :class="[
                  'quick-entry-panel--is-state-3',
                  selectedProjectId === project.id ? 'quick-entry-panel--is-state-4' : '',
                ]"
                @click="
                  selectedProjectId = project.id;
                  billable = Boolean(project.billable);
                  showProjectMenu = false;
                "
              >
                {{ project.title }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <button
        type="button"
        class="quick-entry-panel__button"
        @click="close"
      >
        <X :size="16" />
      </button>
    </div>
  </div>
</template>
