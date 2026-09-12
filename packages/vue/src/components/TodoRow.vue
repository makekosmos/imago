<script setup lang="ts">
import { shallowRef, computed, useTemplateRef, nextTick, ref, watch } from "vue";
import { Calendar as CalendarIcon, DollarSign } from "@lucide/vue";
import ContextMenu from "./ContextMenu.vue";
import ContextMenuItem from "./ContextMenuItem.vue";
import DateChip from "./DateChip.vue";
import type { TodoRowItem, TodoDropPayload, TodoRowUpdate } from "./types";

const props = withDefaults(
  defineProps<{
    todo: TodoRowItem;
    draggable?: boolean;
    editable?: boolean;
  }>(),
  {
    draggable: true,
    editable: true,
  },
);

const emit = defineEmits<{
  complete: [];
  trash: [];
  update: [patch: TodoRowUpdate];
  drop: [payload: TodoDropPayload];
}>();

const isCompleted = computed(() => props.todo.isCompleted || props.todo.isCancelled);

const expanded = shallowRef(false);
const titleDraft = shallowRef(props.todo.title);
const notesDraft = shallowRef(props.todo.notes ?? "");
const dateDraft = shallowRef<string | null>(props.todo.scheduledDate ?? null);
const billableDraft = shallowRef(Boolean(props.todo.billable));
const priceDraft = shallowRef(
  props.todo.price !== null && props.todo.price !== undefined ? String(props.todo.price) : "",
);

watch(
  () => props.todo,
  (next) => {
    if (expanded.value) return;
    titleDraft.value = next.title;
    notesDraft.value = next.notes ?? "";
    dateDraft.value = next.scheduledDate ?? null;
    billableDraft.value = Boolean(next.billable);
    priceDraft.value = next.price !== null && next.price !== undefined ? String(next.price) : "";
  },
  { deep: true },
);

const titleInputRef = useTemplateRef<HTMLInputElement>("titleInput");
const rowRef = ref<HTMLElement>();
const isDragging = shallowRef(false);
const dragSuppressClick = shallowRef(false);

function toggleExpand() {
  if (!props.editable || props.todo.isTrashed) return;
  expanded.value = !expanded.value;
  if (expanded.value) {
    titleDraft.value = props.todo.title;
    notesDraft.value = props.todo.notes ?? "";
    dateDraft.value = props.todo.scheduledDate ?? null;
    billableDraft.value = Boolean(props.todo.billable);
    priceDraft.value =
      props.todo.price !== null && props.todo.price !== undefined ? String(props.todo.price) : "";
    nextTick(() => titleInputRef.value?.focus());
  }
}

function commitTitle() {
  const trimmed = titleDraft.value.trim();
  if (trimmed && trimmed !== props.todo.title) {
    emit("update", { title: trimmed });
  } else {
    titleDraft.value = props.todo.title;
  }
}

function commitNotes() {
  const next = notesDraft.value.trim() || null;
  if (next !== (props.todo.notes ?? null)) {
    emit("update", { notes: next });
  }
}

function commitDate(next: string | null) {
  dateDraft.value = next;
  if (next !== (props.todo.scheduledDate ?? null)) {
    emit("update", { scheduledDate: next });
  }
}

function toggleBillable() {
  billableDraft.value = !billableDraft.value;
  emit("update", { billable: billableDraft.value });
}

function commitPrice() {
  const trimmed = priceDraft.value.trim();
  const parsed = trimmed === "" ? null : Number(trimmed);
  const next = parsed !== null && Number.isFinite(parsed) ? parsed : null;
  if (next !== (props.todo.price ?? null)) {
    emit("update", { price: next });
  }
}

function onExpandKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    expanded.value = false;
  }
}

// ---------------------------------------------------------------------------
// Context menu
// ---------------------------------------------------------------------------

const menuOpen = shallowRef(false);
const menuX = shallowRef(0);
const menuY = shallowRef(0);

function onContextMenu(e: MouseEvent) {
  if (!props.editable) return;
  e.preventDefault();
  menuX.value = e.clientX;
  menuY.value = e.clientY;
  menuOpen.value = true;
}

function closeMenu() {
  menuOpen.value = false;
}

function handleDelete() {
  closeMenu();
  expanded.value = false;
  emit("trash");
}

// ---------------------------------------------------------------------------
// Drag & Drop
// ---------------------------------------------------------------------------

let clone: HTMLElement | null = null;
let ghost: HTMLElement | null = null;
let startX = 0;
let startY = 0;
let lastTarget: TodoDropPayload | null = null;

let rowSnapshot: { id: string; top: number; bottom: number; mid: number }[] = [];

function onRowPointerDown(e: PointerEvent) {
  if (!props.draggable || expanded.value || e.button !== 0) return;

  const ox = e.clientX;
  const oy = e.clientY;
  let started = false;

  const onMove = (me: PointerEvent) => {
    if (!started && (Math.abs(me.clientX - ox) > 4 || Math.abs(me.clientY - oy) > 4)) {
      started = true;
      dragSuppressClick.value = true;
      beginDrag(ox, oy);
    }
    if (started) onDragMove(me);
  };
  const onUp = () => {
    document.removeEventListener("pointermove", onMove);
    if (started) onDragEnd();
  };
  document.addEventListener("pointermove", onMove);
  document.addEventListener("pointerup", onUp, { once: true });
}

function onRowClick(e: MouseEvent) {
  if (dragSuppressClick.value) {
    dragSuppressClick.value = false;
    return;
  }
  if (e.target instanceof HTMLElement && e.target.closest("[data-stop-toggle]")) return;
  toggleExpand();
}

function beginDrag(cx: number, cy: number) {
  if (!rowRef.value) return;

  const rect = rowRef.value.getBoundingClientRect();
  startX = cx;
  startY = cy;
  lastTarget = null;

  rowSnapshot = getSiblingRows().map((el) => {
    const r = el.getBoundingClientRect();
    return { id: el.dataset.todoId!, top: r.top, bottom: r.bottom, mid: r.top + r.height / 2 };
  });

  const copiedRow = rowRef.value.cloneNode(true);
  if (!(copiedRow instanceof HTMLElement)) return;
  clone = copiedRow;
  clone.dataset.dragClone = "";
  Object.assign(clone.style, {
    position: "todo-row__copied-row",
    top: `${rect.top}px`,
    left: `${rect.left}px`,
    width: `${rect.width}px`,
    height: `${rect.height}px`,
    margin: "0",
    pointerEvents: "none",
    zIndex: "9999",
    backgroundColor: "var(--secondary)",
    translate: "0px 0px",
    rotate: "2.5deg",
    transition: "rotate 0.18s ease",
    opacity: "0.9",
  });
  document.body.appendChild(clone);
  document.body.style.cursor = "grabbing";

  ghost = document.createElement("div");
  ghost.dataset.dropGhost = "";
  Object.assign(ghost.style, {
    height: `${rect.height}px`,
    flex: "0 0 auto",
    borderRadius: "var(--radius)",
    border: "1px dashed color-mix(in srgb, var(--ring) 70%, transparent)",
    backgroundColor: "color-mix(in srgb, var(--secondary) 65%, transparent)",
    boxSizing: "border-box",
    pointerEvents: "none",
  });
  rowRef.value.parentElement?.insertBefore(ghost, rowRef.value);
  rowRef.value.style.display = "none";

  isDragging.value = true;
}

function getSiblingRows(): HTMLElement[] {
  return Array.from(document.querySelectorAll<HTMLElement>("[data-todo-id]")).filter(
    (el) => el.dataset.todoId !== props.todo.id && !el.hasAttribute("data-drag-clone"),
  );
}

function findDropTarget(clientY: number): TodoDropPayload | null {
  if (rowSnapshot.length === 0) return null;

  for (const snap of rowSnapshot) {
    if (clientY < snap.mid) {
      return { targetId: snap.id, after: false };
    }
  }

  return { targetId: rowSnapshot[rowSnapshot.length - 1].id, after: true };
}

function positionGhost(target: TodoDropPayload) {
  if (!ghost) return;

  const rows = getSiblingRows();
  const targetEl = rows.find((el) => el.dataset.todoId === target.targetId);
  if (!targetEl) return;

  const refNode = target.after ? targetEl.nextSibling : targetEl;
  const parent = targetEl.parentElement;
  if (!parent) return;
  if (ghost.parentElement === parent && ghost.nextSibling === refNode) return;
  parent.insertBefore(ghost, refNode);
}

function onDragMove(e: PointerEvent) {
  if (!clone) return;

  const dx = e.clientX - startX;
  const dy = e.clientY - startY;
  clone.style.translate = `${dx}px ${dy}px`;
  clone.style.rotate = `${dx >= 0 ? 2.5 : -2.5}deg`;

  const target = findDropTarget(e.clientY);
  if (!target) return;

  if (!lastTarget || lastTarget.targetId !== target.targetId || lastTarget.after !== target.after) {
    lastTarget = target;
    positionGhost(target);
  }
}

function onDragEnd() {
  document.body.style.cursor = "";

  if (clone?.parentElement) {
    clone.parentElement.removeChild(clone);
    clone = null;
  }

  rowRef.value?.style.removeProperty("display");
  if (ghost?.parentElement) {
    ghost.parentElement.removeChild(ghost);
  }
  ghost = null;

  isDragging.value = false;
  rowSnapshot = [];

  if (lastTarget) {
    const payload = lastTarget;
    lastTarget = null;
    emit("drop", payload);
  }
}
</script>

<template>
  <div
    ref="rowRef"
    :data-todo-id="todo.id"
    :class="[
      'group todo-row--is-state',
      isDragging ? 'todo-row--is-state-2' : '',
      expanded ? 'todo-row--is-state-3' : '',
    ]"
    @pointerdown="onRowPointerDown"
    @click="onRowClick"
    @contextmenu="onContextMenu"
  >
    <!-- Row header -->
    <div
      :class="[
        'todo-row--is-state-4',
        draggable && !expanded ? 'todo-row--is-pointer' : 'todo-row--is-state-5',
      ]"
    >
      <!-- Checkbox -->
      <button
        type="button"
        class="todo-row"
        data-stop-toggle
        @pointerdown.stop
        @click.stop="emit('complete')"
      >
        <span
          :class="[
            'todo-row--is-state-6',
            isCompleted ? 'todo-row--is-state-7' : '',
          ]"
        >
          <span v-if="isCompleted" class="todo-row__part-2" />
        </span>
      </button>

      <!-- Title (collapsed) -->
      <div class="todo-row__part-3">
        <div
          :class="[
            'todo-row--is-state-8',
            isCompleted ? 'todo-row--is-foreground' : 'todo-row--is-state-9',
          ]"
        >
          {{ todo.title }}
        </div>
        <div
          v-if="!expanded && todo.notes"
          class="todo-row__part-4"
        >
          {{ todo.notes }}
        </div>
      </div>

      <!-- Trailing chips (collapsed) -->
      <div v-if="!expanded" class="todo-row__part-5">
        <span
          v-if="todo.billable"
          class="todo-row__part-6"
        >
          <DollarSign :size="16" />
          <span v-if="todo.price">{{ todo.price }}</span>
        </span>
        <span
          v-if="todo.scheduledDate"
          class="todo-row__part-7"
        >
          <CalendarIcon :size="16" />
          {{ todo.scheduledDate }}
        </span>
      </div>

      <slot />
    </div>

    <!-- Expanded editor -->
    <div
      v-if="expanded"
      class="todo-row__part-8"
      data-stop-toggle
      @click.stop
      @pointerdown.stop
      @keydown="onExpandKeyDown"
    >
      <input
        ref="titleInput"
        type="text"
        :value="titleDraft"
        class="todo-row__title-input"
        @input="titleDraft = ($event.target as HTMLInputElement).value"
        @blur="commitTitle"
        @keydown.enter.prevent="commitTitle"
      />

      <textarea
        :value="notesDraft"
        :rows="2"
        placeholder="Заметки"
        class="todo-row__textarea"
        @input="notesDraft = ($event.target as HTMLTextAreaElement).value"
        @blur="commitNotes"
      />

      <div class="todo-row__part-11">
        <DateChip :value="dateDraft" placeholder="Без даты" @update:value="commitDate" />

        <button
          type="button"
          :class="[
            'todo-row--is-state-10',
            billableDraft
              ? 'todo-row--is-surface'
              : 'todo-row--is-state-11',
          ]"
          @click="toggleBillable"
        >
          <DollarSign :size="16" />
          <span>Оплачиваемая</span>
        </button>

        <label
          v-if="billableDraft"
          class="todo-row__part-12"
        >
          <input
            type="number"
            inputmode="decimal"
            min="0"
            step="0.01"
            placeholder="Цена"
            :value="priceDraft"
            class="todo-row__input"
            @input="priceDraft = ($event.target as HTMLInputElement).value"
            @blur="commitPrice"
          />
        </label>
      </div>
    </div>
  </div>

  <ContextMenu :open="menuOpen" :x="menuX" :y="menuY" @close="closeMenu">
    <ContextMenuItem destructive @click="handleDelete"> Удалить </ContextMenuItem>
  </ContextMenu>
</template>
