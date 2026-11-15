<script lang="ts" setup>
import {
  computed,
  inject,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  type Ref,
} from "vue";
import {
  AkChevronDownSmall,
  AkChevronUpSmall,
  LuCopy,
  LuEraser,
} from "@kalimahapps/vue-icons";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import { convertArrayOfStringsToContent } from "@agdb-studio/common/src/composables/content/utils";
import type { TAB } from "../../composables/types";
import { useQueryStore } from "../../composables/queryStore";
import { useQueryExecution } from "../../composables/queryExecution";
import QueryRow from "./QueryRow.vue";
import { getQueryStepLabel } from "../../mock/queryApiMock";
import { copyRows } from "../../utils/queryRowClipboard";
import { notifyCopyResult } from "../../utils/rowCopyNotification";

const queryStore = useQueryStore();

const queryId = inject<Ref<string>>("queryId");
const tab = inject<Ref<TAB>>("activeTab");

const query = computed(() => {
  if (!queryId?.value) return null;
  return queryStore.getQuery(queryId.value)?.value;
});

const rows = computed(() => {
  if (!query.value) return [];
  /* v8 ignore next -- @preserve */
  return query.value.rows[tab?.value ?? "exec"] ?? [];
});

const isRunning = computed(() => query.value?.isRunning ?? false);

const { runLabel, canRun, runOrStopQuery } = useQueryExecution(
  queryId,
  tab,
  rows,
  isRunning,
);

// Empty rows are not sent, so the server's result indexes only count the non-empty ones.
const resultIndexes = computed(() => {
  let next = 0;
  return rows.value.map((row) => (row.steps.length > 0 ? next++ : undefined));
});
const nonEmptyRows = computed(() =>
  rows.value.filter((row) => row.steps.length > 0),
);

// Hiding is remembered per tab and only applies when there is more than one row.
const hiddenTabs = ref<Partial<Record<TAB, boolean>>>({});
const canHide = computed(() => rows.value.length > 1);
const rowsHidden = computed(
  () => !!hiddenTabs.value[tab?.value ?? "exec"] && canHide.value,
);
const setRowsHidden = (hidden: boolean): void => {
  hiddenTabs.value[tab?.value ?? "exec"] = hidden;
};
const hiddenCount = computed(() => {
  const count = nonEmptyRows.value.length;
  return `${count} ${count === 1 ? "query" : "queries"}`;
});
const hiddenSummary = computed(() =>
  nonEmptyRows.value
    .map((row) =>
      row.steps.map((step) => getQueryStepLabel(step.type)).join("."),
    )
    .join(" | "),
);
const hasInvalidSteps = computed(() =>
  rows.value.some((row) => row.steps.some((step) => step.invalid)),
);

const dragIndex = ref<number>();
const dropTarget = ref<{ index: number; after: boolean }>();

const moveRow = (fromIndex: number, toIndex: number): void => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.moveQueryRow(
    queryId.value,
    tab?.value ?? "exec",
    fromIndex,
    toIndex,
  );
};

const onDragOver = (event: DragEvent, index: number): void => {
  if (dragIndex.value === undefined) return;
  event.preventDefault();
  /* v8 ignore if -- @preserve */
  if (!(event.currentTarget instanceof HTMLElement)) return;
  // Measure from the row itself; offsetY is relative to whatever child is under the pointer.
  const rect = event.currentTarget.getBoundingClientRect();
  dropTarget.value = {
    index,
    after: event.clientY - rect.top > rect.height / 2,
  };
};

const onDragLeave = (event: DragEvent): void => {
  const container = event.currentTarget;
  const next = event.relatedTarget;
  // Only clear when the pointer leaves the rows entirely, not when it crosses between rows.
  if (
    container instanceof HTMLElement &&
    next instanceof Node &&
    container.contains(next)
  ) {
    return;
  }
  dropTarget.value = undefined;
};

const onDrop = (): void => {
  if (dragIndex.value !== undefined && dropTarget.value) {
    const { index, after } = dropTarget.value;
    let toIndex = after ? index + 1 : index;
    // Removing the dragged row first shifts every later row up by one.
    if (dragIndex.value < toIndex) toIndex -= 1;
    moveRow(dragIndex.value, toIndex);
  }
  onDragEnd();
};

const onDragEnd = (): void => {
  dragIndex.value = undefined;
  dropTarget.value = undefined;
};

// Multi-row selection, by row id so it survives reordering.
const selectedIds = ref<string[]>([]);
const anchorId = ref<string>();
const selectedRows = computed(() =>
  rows.value.filter((row) => selectedIds.value.includes(row.id)),
);
// Rows removed elsewhere (e.g. by clearing) drop out of the selection.
watch(rows, (current) => {
  const ids = new Set(current.map((row) => row.id));
  selectedIds.value = selectedIds.value.filter((id) => ids.has(id));
});
watch(
  () => tab?.value,
  () => clearSelection(),
);

const clearSelection = (): void => {
  selectedIds.value = [];
  anchorId.value = undefined;
};

const selectRow = (index: number, event: MouseEvent): void => {
  const row = rows.value[index];
  /* v8 ignore if -- @preserve */
  if (!row) return;
  const anchorIndex = rows.value.findIndex((r) => r.id === anchorId.value);
  if (event.shiftKey && anchorIndex >= 0) {
    const [from, to] = [anchorIndex, index].sort((a, b) => a - b);
    selectedIds.value = rows.value.slice(from, to + 1).map((r) => r.id);
    return;
  }
  if (event.ctrlKey || event.metaKey) {
    selectedIds.value = selectedIds.value.includes(row.id)
      ? selectedIds.value.filter((id) => id !== row.id)
      : [...selectedIds.value, row.id];
  } else {
    selectedIds.value =
      selectedIds.value.length === 1 && selectedIds.value[0] === row.id
        ? []
        : [row.id];
  }
  anchorId.value = row.id;
};

const copySelection = async (): Promise<void> => {
  const toCopy = selectedRows.value.filter((row) => row.steps.length);
  /* v8 ignore if -- @preserve */
  if (!toCopy.length) return;
  notifyCopyResult(
    await copyRows(toCopy.map((row) => row.steps)),
    toCopy.length,
  );
};

const copyAll = async (): Promise<void> => {
  notifyCopyResult(
    await copyRows(nonEmptyRows.value.map((row) => row.steps)),
    nonEmptyRows.value.length,
  );
};

const removeSelection = (): void => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.deleteQueryRows(
    queryId.value,
    tab?.value ?? "exec",
    selectedIds.value,
  );
  clearSelection();
};

const { openModal } = useModal();
// A single row has its own clear button.
const canClear = computed(() => rows.value.length > 1);
const clearAll = (): void => {
  openModal({
    header: "Clear all queries",
    content: convertArrayOfStringsToContent(
      [
        `Are you sure you want to clear all queries in ${tab?.value ?? "exec"}?`,
      ],
      { emphasizedWords: ["clear"] },
    ),
    onConfirm: () => {
      /* v8 ignore if -- @preserve */
      if (!queryId?.value) return false;
      // A run started while the dialog was open keeps its rows.
      if (isRunning.value) return true;
      queryStore.setQueryRows(queryId.value, tab?.value ?? "exec", []);
      clearSelection();
      return true;
    },
  });
};

const builderElement = ref<HTMLElement>();
const isTextField = (target: EventTarget | null): boolean =>
  target instanceof HTMLInputElement ||
  target instanceof HTMLTextAreaElement ||
  (target instanceof HTMLElement && target.isContentEditable);

/** Ctrl/Cmd+C copies, Delete removes and Esc clears the row selection. */
const onKeydown = (event: KeyboardEvent): void => {
  if (!selectedIds.value.length || isTextField(event.target)) return;
  if (event.key === "Escape") {
    clearSelection();
  } else if (
    (event.ctrlKey || event.metaKey) &&
    event.key.toLowerCase() === "c" &&
    !window.getSelection()?.toString()
  ) {
    event.preventDefault();
    void copySelection();
  } else if (event.key === "Delete" && !isRunning.value) {
    event.preventDefault();
    removeSelection();
  }
};

// A click anywhere outside the builder ends the selection.
const onPointerDown = (event: PointerEvent): void => {
  if (
    selectedIds.value.length &&
    event.target instanceof Node &&
    !builderElement.value?.contains(event.target)
  ) {
    clearSelection();
  }
};
onMounted(() => document.addEventListener("pointerdown", onPointerDown));
onBeforeUnmount(() =>
  document.removeEventListener("pointerdown", onPointerDown),
);

const addRow = (): void => {
  /* v8 ignore if -- @preserve */
  if (!queryId?.value) return;
  queryStore.addQueryRow(queryId.value, tab?.value ?? "exec");
};
</script>

<template>
  <div ref="builderElement" class="query-builder" @keydown="onKeydown">
    <div class="query-rows" :class="[tab]" @dragleave="onDragLeave">
      <button
        v-if="rowsHidden"
        type="button"
        class="hidden-summary"
        data-testid="query-hidden-summary"
        :class="{ invalid: hasInvalidSteps }"
        title="Show queries"
        @click="setRowsHidden(false)"
      >
        <span class="hidden-count">{{ hiddenCount }}</span>
        <span class="hidden-preview">{{ hiddenSummary }}</span>
      </button>
      <template v-else>
        <QueryRow
          v-for="(row, index) in rows"
          :key="row.id"
          :row="row"
          :index="index"
          :result-index="resultIndexes[index]"
          :show-index="rows.length > 1"
          :can-move="rows.length > 1 && !isRunning"
          :can-remove="rows.length > 1"
          :can-add="index === rows.length - 1"
          :is-running="isRunning"
          :selected="selectedIds.includes(row.id)"
          :selection-count="selectedIds.length"
          :class="{
            dragging: dragIndex === index,
            'drop-before': dropTarget?.index === index && !dropTarget.after,
            'drop-after': dropTarget?.index === index && dropTarget.after,
          }"
          @add-row="addRow"
          @select="(event: MouseEvent) => selectRow(index, event)"
          @copy-selection="copySelection"
          @remove-selection="removeSelection"
          @move="(toIndex: number) => moveRow(index, toIndex)"
          @drag-start="dragIndex = index"
          @dragover="(event: DragEvent) => onDragOver(event, index)"
          @drop.prevent="onDrop"
          @dragend="onDragEnd"
        />
      </template>
    </div>
    <div class="builder-actions">
      <button
        type="button"
        class="button run-query-button"
        :class="[tab === 'exec_mut' ? 'button-warning' : 'button-primary']"
        :disabled="!canRun && !isRunning"
        @click="runOrStopQuery"
      >
        {{ runLabel }}
      </button>
      <div
        v-if="canHide || nonEmptyRows.length > 1 || canClear"
        class="builder-secondary"
      >
        <button
          v-if="canHide"
          type="button"
          class="button button-transparent toggle-rows-button"
          :aria-expanded="!rowsHidden"
          @click="setRowsHidden(!rowsHidden)"
        >
          <component :is="rowsHidden ? AkChevronDownSmall : AkChevronUpSmall" />
          {{ rowsHidden ? "Show" : "Hide" }} queries
        </button>
        <button
          v-if="nonEmptyRows.length > 1"
          type="button"
          class="button button-transparent builder-action-button copy-all-button"
          data-testid="query-copy-all"
          title="Copy all queries as Rust code, one per line"
          aria-label="Copy all queries"
          @click="copyAll"
        >
          <LuCopy />
        </button>
        <button
          v-if="canClear"
          type="button"
          class="button button-transparent builder-action-button clear-all-button"
          data-testid="query-clear-all"
          title="Remove all queries in this tab"
          :disabled="isRunning"
          @click="clearAll"
        >
          <LuEraser />
        </button>
      </div>
    </div>
  </div>
</template>

<style lang="less" scoped>
.query-builder {
  width: 100%;
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}
.builder-actions {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  flex-shrink: 0;
  min-width: 7rem;
}
.run-query-button {
  // Matches the height of a single row (row min-height + the rows' border) and stays there as rows are added.
  height: calc(2rem + 2px);
}
// The toggle, then small icon buttons on the same line.
.builder-secondary {
  display: flex;
  align-items: center;
  .toggle-rows-button {
    margin-right: auto;
  }
}
.toggle-rows-button,
.builder-action-button {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.2rem;
  font-size: 0.8rem;
  padding: 0.1rem 0.3rem;
  opacity: 0.7;
  &:hover:not(:disabled) {
    opacity: 1;
  }
}
.clear-all-button:hover:not(:disabled) {
  color: var(--error-color);
}
.hidden-summary {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2rem;
  padding: 0.2rem 0.5rem;
  border: none;
  background: none;
  color: var(--color-text);
  font: inherit;
  text-align: left;
  cursor: pointer;
  &.invalid .hidden-count {
    color: var(--error-color);
  }
}
.hidden-count {
  flex-shrink: 0;
  font-weight: 500;
}
.hidden-preview {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  opacity: 0.6;
}
// Applied to QueryRow's root element while reordering.
.dragging {
  opacity: 0.4;
}
.drop-before {
  box-shadow: inset 0 2px 0 var(--color-text);
}
.drop-after {
  box-shadow: inset 0 -2px 0 var(--color-text);
}
.query-rows {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-text);
  border-radius: 0.25rem;
  box-sizing: border-box;
  transition: background-color 0.4s ease;
  min-height: calc(2rem + 2px);
  &.context {
    background-color: var(--color-background-soft);
  }
  &.exec_mut {
    background-color: var(--orange-background);
  }
}
</style>
