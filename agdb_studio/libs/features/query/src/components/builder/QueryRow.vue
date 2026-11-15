<script lang="ts" setup>
import { computed, inject, nextTick, ref, type Ref } from "vue";
import type {
  CopiedStep,
  QueryRow,
  QueryType,
  TAB,
} from "../../composables/types";
import { useQueryStore } from "../../composables/queryStore";
import QueryStep from "./QueryStep.vue";
import QueryStepInput from "./QueryStepInput.vue";
import ColorSwatchPicker from "../graph/ColorSwatchPicker.vue";
import { useGraphPalette } from "../../composables/graphPalette";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import {
  ClAddRow,
  ClTrashFull,
  LuClipboardPaste,
  LuCopy,
  LuEraser,
  LuGripVertical,
} from "@kalimahapps/vue-icons";
import {
  copySteps,
  pastedQuery,
  readCopiedRows,
} from "../../utils/queryRowClipboard";
import {
  notifyCopyResult,
  notifyPasteFailure,
} from "../../utils/rowCopyNotification";
import type { PaletteSlot } from "../../utils/graphColorRules";

const props = defineProps<{
  row: QueryRow;
  /** Position of the row in the tab, used for reordering. */
  index: number;
  /** Position of this row's result in the response; undefined for empty rows, which are not sent. */
  resultIndex?: number;
  showIndex: boolean;
  canMove: boolean;
  canRemove: boolean;
  canAdd: boolean;
  isRunning: boolean;
  /** Part of the multi-row selection; copy and remove then act on all selected rows. */
  selected?: boolean;
  selectionCount?: number;
}>();

const emit = defineEmits<{
  (e: "add-row"): void;
  (e: "drag-start"): void;
  (e: "move", toIndex: number): void;
  (e: "select", event: MouseEvent): void;
  (e: "copy-selection"): void;
  (e: "remove-selection"): void;
}>();

// Copy and remove act on the whole selection when this row is part of one.
const inSelection = computed(
  () => !!props.selected && (props.selectionCount ?? 0) > 1,
);

const rowElement = ref<HTMLElement>();
const handleElement = ref<HTMLElement>();
// The input only spans its text; clicking the empty rest of the row focuses it instead.
const stepInput = ref<InstanceType<typeof QueryStepInput>>();

const onDragStart = (event: DragEvent): void => {
  /* v8 ignore else -- @preserve */
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", String(props.index));
    // Drag the whole row, not just the handle.
    if (rowElement.value) {
      event.dataTransfer.setDragImage(rowElement.value, 0, 0);
    }
  }
  emit("drag-start");
};

const onHandleKeydown = (event: KeyboardEvent): void => {
  const offset = { ArrowUp: -1, ArrowDown: 1 }[event.key];
  if (!offset) return;
  event.preventDefault();
  emit("move", props.index + offset);
  // The row's element is moved in the DOM, which drops focus; keep it on the handle.
  void nextTick(() => handleElement.value?.focus());
};

const queryStore = useQueryStore();

const queryId = inject<Ref<string>>("queryId");
const tab = inject<Ref<TAB>>("activeTab");

const steps = computed(() => props.row.steps);

// Elements this row returns can be given a colour in the graph.
const palette = useGraphPalette();
const rowColor = computed(() => {
  if (!queryId?.value) return undefined;
  return queryStore.getQuery(queryId.value)?.value.colors?.rows[props.row.id];
});
const setRowColor = (slot: PaletteSlot | undefined): void => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.setRowColor(queryId.value, props.row.id, slot);
};

const clearSteps = (): void => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.clearQuerySteps(queryId.value, tab?.value ?? "exec", props.row.id);
};

const removeRow = (): void => {
  if (inSelection.value) {
    emit("remove-selection");
    return;
  }
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.deleteQueryRow(queryId.value, tab?.value ?? "exec", props.row.id);
};

const pasteLabel = computed(() =>
  steps.value.length
    ? "Paste a copied query below this one"
    : "Paste a copied query here",
);

const copyLabel = computed(() =>
  inSelection.value
    ? `Copy the ${props.selectionCount} selected queries as Rust code`
    : "Copy this query as Rust code",
);

const copyRow = async (): Promise<void> => {
  if (inSelection.value) {
    emit("copy-selection");
    return;
  }
  notifyCopyResult(await copySteps(steps.value));
};

/** Pastes one row into or after this one, or several rows after it. */
const insertRows = (rows: CopiedStep[][]): void => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  const currentTab = tab?.value ?? "exec";
  const [first] = rows;
  if (rows.length === 1 && first) {
    queryStore.pasteQuerySteps(queryId.value, currentTab, props.row.id, first);
  } else {
    queryStore.insertQueryRows(queryId.value, currentTab, props.row.id, rows);
  }
};

const pasteRow = async (): Promise<void> => {
  const read = await readCopiedRows();
  if ("rows" in read) {
    insertRows(read.rows);
  } else {
    notifyPasteFailure(read);
  }
};

/**
 * Ctrl/Cmd+V in the row (outside argument fields) pastes query code, e.g.
 * from Rust; other text pastes as usual.
 */
const onPaste = (event: ClipboardEvent): void => {
  // Argument fields take any text, e.g. a key such as "user.name".
  const target = event.target;
  if (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement
  ) {
    return;
  }
  if (props.isRunning) return;
  const copied = pastedQuery(event.clipboardData?.getData("text/plain") ?? "");
  if (!copied) return;
  event.preventDefault();
  insertRows(copied);
};

const addStep = (stepType: QueryType) => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.addQueryStep(queryId.value, tab?.value ?? "exec", props.row.id, {
    id: `step-${crypto.randomUUID()}`,
    type: stepType,
  });
};
</script>

<template>
  <div
    ref="rowElement"
    class="query-input"
    :class="{ selected }"
    data-testid="query-row"
    @paste="onPaste"
  >
    <button
      v-if="canMove"
      ref="handleElement"
      type="button"
      class="button button-transparent row-action drag-handle"
      title="Drag to reorder, or use the arrow keys"
      aria-label="Reorder query, use up and down arrow keys"
      draggable="true"
      @dragstart="onDragStart"
      @keydown="onHandleKeydown"
    >
      <LuGripVertical />
    </button>
    <button
      v-if="showIndex"
      type="button"
      class="row-index"
      data-testid="query-row-index"
      :aria-pressed="!!selected"
      :title="`${
        resultIndex === undefined ? 'Empty queries are skipped' : 'Result index'
      }. Click to select, Ctrl+click to add, Shift+click for a range`"
      @click="emit('select', $event)"
    >
      {{ resultIndex }}
    </button>
    <ColorSwatchPicker
      v-if="tab !== 'context' && steps.length > 0"
      class="row-color"
      data-testid="query-row-color"
      :palette="palette"
      :model-value="rowColor"
      label="Colour of this query's elements in the graph"
      @update:model-value="setRowColor"
    />
    <!-- Steps wrap within their own column, so the handle, index and actions stay put. -->
    <div class="row-steps" @click.self="stepInput?.focus()">
      <div v-for="step in steps" :key="step.id" class="query-step-wrapper">
        <span class="separator">.</span>
        <QueryStep :step="step" />
      </div>
      <QueryStepInput
        ref="stepInput"
        :prev-step="steps.length > 0 ? steps[steps.length - 1] : undefined"
        @confirm-step="addStep"
      />
      <!-- Right after the query (and anything being typed) so it reads as acting on it. -->
      <button
        v-if="steps.length && !isRunning"
        type="button"
        class="button button-transparent row-action remove-button"
        title="Clear all steps of this query"
        aria-label="Clear all steps of this query"
        @click="clearSteps"
      >
        <LuEraser />
      </button>
    </div>
    <!-- Remove stays right-most so the trash icons line up across rows. -->
    <div class="row-actions">
      <button
        v-if="steps.length"
        type="button"
        class="button button-transparent row-action copy-row-button"
        :title="copyLabel"
        :aria-label="copyLabel"
        @click="copyRow"
      >
        <LuCopy />
      </button>
      <button
        type="button"
        class="button button-transparent row-action paste-row-button"
        :title="pasteLabel"
        :aria-label="pasteLabel"
        :disabled="isRunning"
        @click="pasteRow"
      >
        <LuClipboardPaste />
      </button>
      <button
        v-if="canAdd"
        type="button"
        class="button button-transparent row-action add-row-button"
        title="Add another query to run in the same transaction"
        aria-label="Add another query to run in the same transaction"
        :disabled="isRunning"
        @click="emit('add-row')"
      >
        <ClAddRow />
      </button>
      <SpinnerIcon v-if="isRunning" />
      <template v-else>
        <button
          v-if="canRemove"
          type="button"
          class="button button-transparent row-action row-action-danger remove-row-button"
          :title="
            inSelection
              ? `Remove the ${selectionCount} selected queries`
              : 'Remove this query'
          "
          :aria-label="
            inSelection
              ? `Remove the ${selectionCount} selected queries`
              : 'Remove this query'
          "
          @click="removeRow"
        >
          <ClTrashFull />
        </button>
      </template>
    </div>
  </div>
</template>

<style lang="less" scoped>
.query-input {
  padding: 0.2rem;
  box-sizing: border-box;
  display: flex;
  gap: 0.2rem;
  // Handle, index and actions stay on the first line when steps wrap.
  align-items: flex-start;
  min-height: 2rem;
  & + & {
    border-top: 1px dashed var(--color-border);
  }
  &.selected {
    background-color: var(--color-background-mute);
    box-shadow: inset 3px 0 0 var(--blue-2);
  }
}
.row-steps {
  flex: 1;
  cursor: text;
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.2rem;
}
.drag-handle {
  cursor: grab;
  padding: 0.2rem 0;
}
.drag-handle,
.row-index,
.row-actions {
  // Vertically centre on the first line of steps.
  min-height: 1.6rem;
  align-self: flex-start;
}
.row-index {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  min-width: 1.2rem;
  text-align: right;
  font-size: 0.75rem;
  opacity: 0.5;
  user-select: none;
  // Reset the button look; it reads as the plain index.
  padding: 0;
  border: none;
  background: none;
  color: inherit;
  cursor: pointer;
  &:hover,
  &:focus-visible,
  &[aria-pressed="true"] {
    opacity: 1;
  }
}
.row-color {
  min-height: 1.6rem;
  align-self: flex-start;
}
.row-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.1rem;
}
.row-action {
  font-size: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.2rem;
  opacity: 0.6;
  &:hover:not(:disabled),
  &:focus-visible {
    opacity: 1;
  }
}
.row-action-danger:hover:not(:disabled) {
  color: var(--error-color);
}
</style>
