<script lang="ts" setup>
import type { Components } from "@agnesoft/agdb_api";
import { computed, ref, watch } from "vue";
import { LuPencil } from "@kalimahapps/vue-icons";
import InlineFieldEditor from "./InlineFieldEditor.vue";
import type {
  GraphElement,
  QueryStepFieldValue,
} from "../../composables/types";
import { isNode, type PanelMode } from "../../composables/graphActions";
import { dbValueToString } from "../../utils/graphDataUtils";
import { linkEndpointId } from "../../utils/graphStyle";
import {
  EDITABLE_VALUE_TYPES,
  NESTED_TYPE,
  dbValueToField,
  dbValueTypeLabel,
  fieldToDbValue,
  validateField,
} from "../../utils/dbValueFields";

type EditRow = {
  id: number;
  key: QueryStepFieldValue;
  value: QueryStepFieldValue;
  /** Original key-value and its initial fields, to keep untouched parts exact. */
  original?: {
    kv: Components.Schemas.DbKeyValue;
    key: QueryStepFieldValue;
    value: QueryStepFieldValue;
  };
};

const props = withDefaults(
  defineProps<{
    element: GraphElement;
    mode?: PanelMode;
    saving?: boolean;
  }>(),
  { mode: "view", saving: false },
);

const emit = defineEmits<{
  close: [];
  /** Retry loading the stored values after a failure. */
  retry: [];
  edit: [];
  cancel: [];
  /** Whether the form has changes that would be lost by closing it. */
  dirty: [dirty: boolean];
  /** `original` is the copy of the values taken when editing started. */
  save: [
    values: Components.Schemas.DbKeyValue[],
    original: Components.Schemas.DbKeyValue[],
  ];
}>();

const title = computed(
  () =>
    `${isNode(props.element) ? "Node" : "Edge"} ${props.element.id}${
      props.element.alias ? ` (${props.element.alias})` : ""
    }`,
);

const endpoints = computed(() =>
  isNode(props.element)
    ? undefined
    : {
        from: linkEndpointId(props.element.source),
        to: linkEndpointId(props.element.target),
      },
);

/** Editing needs the full stored values, or saving could drop some. */
const valuesReady = computed(
  () =>
    props.element.valuesState !== "loading" &&
    props.element.valuesState !== "error",
);
const queryOnlyValues = computed(() => props.element.queryOnlyValues ?? []);

let nextRowId = 0;
const rows = ref<EditRow[]>([]);
const error = ref<string>();
/** Values when editing started; later result updates must not change it. */
let baseline: Components.Schemas.DbKeyValue[] = [];
/** Whether the form was filled from the full stored values. */
const baselineReady = ref(false);

const resetRows = () => {
  error.value = undefined;
  baselineReady.value = valuesReady.value;
  baseline = [...props.element.rawValues];
  rows.value = baseline.map((kv) => {
    const key = dbValueToField(kv.key);
    const value = dbValueToField(kv.value);
    return {
      id: nextRowId++,
      key: { ...key },
      value: { ...value },
      original: { kv, key, value },
    };
  });
};

// Element objects are rebuilt whenever any result changes (e.g. after expanding
// a node), so only a different element or mode resets the form.
// Separate sources are compared one by one; a getter returning a new array
// would fire on every change.
watch([() => props.element.id, () => props.mode], () => resetRows(), {
  immediate: true,
});
// Fill the form once the stored values first arrive. Later reloads (e.g. a
// new run while editing) keep the edits and the original copy.
watch(valuesReady, (ready) => {
  if (ready && !baselineReady.value) resetRows();
});

const showForm = computed(
  () => props.mode === "edit" && (valuesReady.value || baselineReady.value),
);

const addRow = () => {
  rows.value.push({
    id: nextRowId++,
    key: { selectedOption: "string", value: "" },
    value: { selectedOption: "string", value: "" },
  });
};

const removeRow = (id: number) => {
  rows.value = rows.value.filter((row) => row.id !== id);
};

/** Validates the rows and returns an error message, if any. */
const validate = (): string | undefined => {
  const seen = new Set<string>();
  for (const [index, row] of rows.value.entries()) {
    const position = `Row ${index + 1}`;
    if (!isKeyLocked(row) && !(row.key.value ?? "").trim()) {
      return `${position}: key is required`;
    }
    // Untouched parts are sent back exactly as loaded, so only edits are
    // validated (e.g. a stored u64 above 2^53 must not block saving).
    const keyError = keyUntouched(row) ? undefined : validateField(row.key);
    if (keyError) return `${position} key: ${keyError}`;
    const valueError = valueUntouched(row)
      ? undefined
      : validateField(row.value);
    if (valueError) return `${position} value: ${valueError}`;
    const keyId = JSON.stringify(rowToKeyValue(row).key);
    if (seen.has(keyId)) return `${position}: duplicate key`;
    seen.add(keyId);
  }
  return undefined;
};

const sameField = (a: QueryStepFieldValue, b: QueryStepFieldValue) =>
  a.selectedOption === b.selectedOption && a.value === b.value;

/** Nested keys and values have no editor; the original is kept as is. */
const isKeyLocked = (row: EditRow): boolean =>
  row.original?.key.selectedOption === NESTED_TYPE;
const isValueLocked = (row: EditRow): boolean =>
  row.original?.value.selectedOption === NESTED_TYPE;

const keyUntouched = (row: EditRow): boolean =>
  !!row.original && (isKeyLocked(row) || sameField(row.key, row.original.key));
const valueUntouched = (row: EditRow): boolean =>
  !!row.original &&
  (isValueLocked(row) || sameField(row.value, row.original.value));

/**
 * Untouched keys and values keep their exact original `DbValue`, so editing
 * only the key (or only the value) never re-encodes the other part.
 */
const rowToKeyValue = (row: EditRow): Components.Schemas.DbKeyValue => ({
  key:
    row.original && keyUntouched(row)
      ? row.original.kv.key
      : fieldToDbValue(row.key),
  value:
    row.original && valueUntouched(row)
      ? row.original.kv.value
      : fieldToDbValue(row.value),
});

/** Rows changed, added or removed since editing started. */
const dirty = computed(
  () =>
    showForm.value &&
    (rows.value.length !== baseline.length ||
      rows.value.some(
        (row) => !row.original || !keyUntouched(row) || !valueUntouched(row),
      )),
);

/**
 * In-place editing in the values table: one key or value at a time, or a
 * new value. Each change is saved on its own; the form is for bulk edits.
 */
type InlineEdit = {
  /** Index into `rawValues`, or "new" for a value being added. */
  index: number | "new";
  part: "key" | "value";
  /** Edited in place by the inline editors, which get them as props. */
  key: QueryStepFieldValue;
  value: QueryStepFieldValue;
};
const inline = ref<InlineEdit>();
const inlineError = ref<string>();

const canInlineEdit = computed(
  () => props.mode === "view" && valuesReady.value && !props.saving,
);

const isNested = (value: Components.Schemas.DbValue): boolean =>
  dbValueToField(value).selectedOption === NESTED_TYPE;

const startInline = (index: number, part: "key" | "value") => {
  const kv = props.element.rawValues[index];
  if (!kv || !canInlineEdit.value || isNested(kv[part])) return;
  inlineError.value = undefined;
  inline.value = {
    index,
    part,
    key: dbValueToField(kv.key),
    value: dbValueToField(kv.value),
  };
};

const startAdd = () => {
  /* v8 ignore if -- @preserve */
  if (!canInlineEdit.value) return;
  inlineError.value = undefined;
  inline.value = {
    index: "new",
    part: "key",
    key: { selectedOption: "string", value: "" },
    value: { selectedOption: "string", value: "" },
  };
};

const cancelInline = () => {
  inline.value = undefined;
  inlineError.value = undefined;
};

const isEditing = (index: number, part: "key" | "value") =>
  inline.value?.index === index && inline.value.part === part;

const inlineDirty = computed(() => {
  const edit = inline.value;
  if (!edit) return false;
  if (edit.index === "new") return !!(edit.key.value || edit.value.value);
  const kv = props.element.rawValues[edit.index];
  return !!kv && !sameField(edit[edit.part], dbValueToField(kv[edit.part]));
});

/** Saves the in-place change, or explains why it cannot be saved. */
const commitInline = () => {
  const edit = inline.value;
  /* v8 ignore if -- @preserve */
  if (!edit) return;
  const values = [...props.element.rawValues];
  const keyTaken = (key: Components.Schemas.DbValue, except?: number) =>
    values.some(
      (kv, i) => i !== except && JSON.stringify(kv.key) === JSON.stringify(key),
    );

  if (edit.index === "new") {
    if (!(edit.key.value ?? "").trim()) {
      inlineError.value = "Key is required";
      return;
    }
    const fieldError = validateField(edit.key) ?? validateField(edit.value);
    if (fieldError) {
      inlineError.value = fieldError;
      return;
    }
    const added = {
      key: fieldToDbValue(edit.key),
      value: fieldToDbValue(edit.value),
    };
    if (keyTaken(added.key)) {
      inlineError.value = "Duplicate key";
      return;
    }
    values.push(added);
  } else {
    const kv = values[edit.index];
    /* v8 ignore if -- @preserve: the row is gone only after a reload */
    if (!kv) return cancelInline();
    if (!inlineDirty.value) return cancelInline();
    const field = edit[edit.part];
    if (edit.part === "key" && !(field.value ?? "").trim()) {
      inlineError.value = "Key is required";
      return;
    }
    const fieldError = validateField(field);
    if (fieldError) {
      inlineError.value = fieldError;
      return;
    }
    const changed = { ...kv, [edit.part]: fieldToDbValue(field) };
    if (edit.part === "key" && keyTaken(changed.key, edit.index)) {
      inlineError.value = "Duplicate key";
      return;
    }
    values[edit.index] = changed;
  }
  cancelInline();
  emit("save", values, props.element.rawValues);
};

/** Leaving the editor (e.g. clicking elsewhere) saves it, like Enter. */
const onInlineFocusOut = (event: FocusEvent) => {
  const cell = event.currentTarget;
  const next = event.relatedTarget;
  if (
    cell instanceof HTMLElement &&
    next instanceof Node &&
    cell.contains(next)
  )
    return;
  // An untouched new value is just dropped.
  if (inline.value?.index === "new" && !inlineDirty.value) {
    cancelInline();
    return;
  }
  commitInline();
};

const removeValue = (index: number) => {
  /* v8 ignore if -- @preserve */
  if (!canInlineEdit.value) return;
  emit(
    "save",
    props.element.rawValues.filter((_, i) => i !== index),
    props.element.rawValues,
  );
};

watch(
  () => props.element.id,
  () => cancelInline(),
);

watch(
  () => dirty.value || inlineDirty.value,
  (value) => emit("dirty", value),
);

/** Esc closes the panel, or leaves edit mode; unsaved edits are kept. */
const onEscape = () => {
  if (props.mode !== "edit") {
    emit("close");
  } else if (dirty.value) {
    error.value = "Unsaved changes: save them or press Cancel to discard.";
  } else {
    emit("cancel");
  }
};

const save = () => {
  error.value = validate();
  if (error.value) return;
  emit("save", rows.value.map(rowToKeyValue), baseline);
};
</script>

<template>
  <aside
    class="element-details-panel"
    data-testid="element-details-panel"
    @keydown.esc.stop="onEscape"
  >
    <header class="panel-header">
      <h3>{{ title }}</h3>
      <button
        v-if="!showForm"
        type="button"
        class="button header-button edit-button"
        data-testid="panel-edit"
        :disabled="!valuesReady"
        :title="
          valuesReady
            ? 'Edit all values in a form'
            : 'Waiting for the stored values'
        "
        aria-label="Edit all values in a form"
        @click="emit('edit')"
      >
        <LuPencil />
      </button>
      <button
        type="button"
        class="button header-button close-button"
        title="Close details"
        data-testid="panel-close"
        @click="emit('close')"
      >
        ×
      </button>
    </header>

    <p v-if="endpoints" class="endpoints" data-testid="panel-endpoints">
      From node <strong>{{ endpoints.from }}</strong> to node
      <strong>{{ endpoints.to }}</strong>
    </p>

    <p
      v-if="props.element.valuesState === 'loading'"
      class="values-note"
      data-testid="panel-values-loading"
    >
      Loading stored values…
    </p>
    <p
      v-else-if="props.element.valuesState === 'error'"
      class="values-note error"
      data-testid="panel-values-error"
    >
      Stored values could not be loaded; showing what the query returned.
      <button
        type="button"
        class="button retry-button"
        data-testid="panel-values-retry"
        @click="emit('retry')"
      >
        Retry
      </button>
    </p>

    <template v-if="!showForm">
      <table
        v-if="props.element.rawValues.length || inline?.index === 'new'"
        class="values-table"
        :class="{ editable: canInlineEdit }"
      >
        <thead>
          <tr>
            <th>Key</th>
            <th>Value</th>
            <th>Type</th>
            <th class="actions-col"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(kv, index) in props.element.rawValues"
            :key="index"
            data-testid="panel-value-row"
          >
            <td
              v-if="isEditing(index, 'key') && inline"
              colspan="3"
              class="editing"
              @focusout="onInlineFocusOut"
            >
              <InlineFieldEditor
                :model-value="inline.key"
                label="Key"
                autofocus
                @commit="commitInline"
                @cancel="cancelInline"
              />
            </td>
            <td
              v-else
              class="key"
              data-testid="panel-key-cell"
              :title="
                canInlineEdit && !isNested(kv.key) ? 'Click to edit' : undefined
              "
              @click="startInline(index, 'key')"
            >
              {{ dbValueToString(kv.key) }}
            </td>
            <template v-if="!isEditing(index, 'key')">
              <td
                v-if="isEditing(index, 'value') && inline"
                colspan="2"
                class="editing"
                @focusout="onInlineFocusOut"
              >
                <InlineFieldEditor
                  :model-value="inline.value"
                  label="Value"
                  autofocus
                  @commit="commitInline"
                  @cancel="cancelInline"
                />
              </td>
              <template v-else>
                <td
                  class="value"
                  data-testid="panel-value-cell"
                  :title="
                    canInlineEdit && !isNested(kv.value)
                      ? 'Click to edit'
                      : undefined
                  "
                  @click="startInline(index, 'value')"
                >
                  {{ dbValueToString(kv.value) }}
                </td>
                <td class="type">{{ dbValueTypeLabel(kv.value) }}</td>
              </template>
            </template>
            <td class="actions-col">
              <button
                v-if="canInlineEdit"
                type="button"
                class="button remove-value-button"
                title="Remove this value"
                aria-label="Remove this value"
                data-testid="panel-remove-value"
                @click="removeValue(index)"
              >
                ×
              </button>
            </td>
          </tr>
          <tr v-if="inline?.index === 'new'" data-testid="panel-new-value-row">
            <td colspan="4" class="editing" @focusout="onInlineFocusOut">
              <div class="new-value">
                <InlineFieldEditor
                  :model-value="inline.key"
                  label="New key"
                  placeholder="key"
                  autofocus
                  @commit="commitInline"
                  @cancel="cancelInline"
                />
                <InlineFieldEditor
                  :model-value="inline.value"
                  label="New value"
                  placeholder="value"
                  @commit="commitInline"
                  @cancel="cancelInline"
                />
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-else class="empty">No values.</p>
      <p v-if="inlineError" class="error" data-testid="panel-inline-error">
        {{ inlineError }}
      </p>
      <p v-if="props.saving" class="values-note" data-testid="panel-saving">
        Saving…
      </p>
      <button
        v-if="canInlineEdit && inline?.index !== 'new'"
        type="button"
        class="button add-button add-value-button"
        data-testid="panel-add-value"
        @click="startAdd"
      >
        + Add value
      </button>
      <template v-if="queryOnlyValues.length">
        <h4 class="section-title">From the query</h4>
        <p class="section-hint">
          Returned by the query but not stored on the element; read-only.
        </p>
        <table class="values-table" data-testid="panel-query-values">
          <tbody>
            <tr
              v-for="(kv, index) in queryOnlyValues"
              :key="index"
              data-testid="panel-query-value-row"
            >
              <td class="key">{{ dbValueToString(kv.key) }}</td>
              <td class="value">{{ dbValueToString(kv.value) }}</td>
              <td class="type">{{ dbValueTypeLabel(kv.value) }}</td>
            </tr>
          </tbody>
        </table>
      </template>
    </template>

    <form v-else class="edit-form" @submit.prevent="save">
      <div
        v-for="row in rows"
        :key="row.id"
        class="edit-row"
        data-testid="panel-edit-row"
      >
        <template v-if="isKeyLocked(row)">
          <span class="type-select locked-type">nested</span>
          <input
            :value="row.key.value"
            class="key-input"
            aria-label="Key"
            title="Nested keys cannot be edited here"
            data-testid="panel-key-locked"
            readonly
          />
        </template>
        <template v-else>
          <select
            v-model="row.key.selectedOption"
            class="type-select"
            aria-label="Key type"
          >
            <option v-for="t in EDITABLE_VALUE_TYPES" :key="t" :value="t">
              {{ t }}
            </option>
          </select>
          <input
            v-model="row.key.value"
            class="key-input"
            placeholder="key"
            aria-label="Key"
            data-testid="panel-key-input"
          />
        </template>
        <template v-if="isValueLocked(row)">
          <span class="type-select locked-type">nested</span>
          <input
            :value="row.value.value"
            class="value-input"
            aria-label="Value"
            title="Nested values cannot be edited here"
            data-testid="panel-value-locked"
            readonly
          />
        </template>
        <template v-else>
          <select
            v-model="row.value.selectedOption"
            class="type-select"
            aria-label="Value type"
            data-testid="panel-value-type"
          >
            <option v-for="t in EDITABLE_VALUE_TYPES" :key="t" :value="t">
              {{ t }}
            </option>
          </select>
          <input
            v-model="row.value.value"
            class="value-input"
            placeholder="value"
            aria-label="Value"
            data-testid="panel-value-input"
          />
        </template>
        <button
          type="button"
          class="button remove-button"
          title="Remove value"
          data-testid="panel-remove-row"
          @click="removeRow(row.id)"
        >
          ×
        </button>
      </div>
      <button
        type="button"
        class="button add-button"
        data-testid="panel-add-row"
        @click="addRow"
      >
        + Add value
      </button>
      <p v-if="error" class="error" data-testid="panel-error">{{ error }}</p>
      <div class="panel-actions">
        <button
          type="button"
          class="button"
          data-testid="panel-cancel"
          @click="emit('cancel')"
        >
          Cancel
        </button>
        <button
          type="submit"
          class="button button-success"
          :disabled="props.saving"
          data-testid="panel-save"
        >
          Save
        </button>
      </div>
    </form>
  </aside>
</template>

<style lang="less" scoped>
.element-details-panel {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  bottom: 0.5rem;
  width: min(28rem, calc(100% - 1rem));
  overflow-y: auto;
  padding: 0.75rem 1rem;
  background-color: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  box-shadow: 0 8px 16px 0 rgba(0, 0, 0, 0.2);
  z-index: 2;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.panel-header {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  h3 {
    font-weight: bold;
    margin-right: auto;
  }
}
.header-button,
.remove-button {
  padding: 0 0.5rem;
  line-height: 1.5rem;
}
.edit-button {
  display: flex;
  align-items: center;
  height: 1.5rem;
}
.endpoints {
  color: var(--color-text-muted);
}
.values-table {
  width: 100%;
  border-collapse: collapse;
  th,
  td {
    text-align: left;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--color-border);
    word-break: break-word;
  }
  th {
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }
  .type {
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }
  .actions-col {
    width: 1.5rem;
    padding: 0;
  }
  &.editable {
    .key[title],
    .value[title] {
      cursor: text;
      &:hover {
        background-color: var(--color-background-soft);
      }
    }
  }
  .editing {
    padding: 0.15rem 0.25rem;
  }
  // Only on the hovered row, so the table stays calm.
  .remove-value-button {
    visibility: hidden;
    padding: 0 0.3rem;
    line-height: 1.2rem;
    border: none;
    background: none;
    color: var(--color-text-muted);
    &:hover {
      color: var(--error-color);
    }
  }
  tr:hover .remove-value-button,
  .remove-value-button:focus-visible {
    visibility: visible;
  }
}
.new-value {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.25rem;
}
.empty {
  color: var(--color-text-muted);
}
.edit-form {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.edit-row {
  display: grid;
  grid-template-columns: 5.5rem 1fr 5.5rem 1fr auto;
  gap: 0.25rem;
  align-items: center;
  input,
  select {
    min-width: 0;
  }
  .locked-type {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }
  input[readonly] {
    color: var(--color-text-muted);
    cursor: not-allowed;
  }
}
.values-note {
  font-size: 0.85rem;
  color: var(--color-text-muted);
  &.error {
    color: var(--error-color);
  }
}
.retry-button {
  margin-left: 0.25rem;
}
.section-title {
  margin: 0.75rem 0 0;
  font-size: 0.9rem;
}
.section-hint {
  margin: 0;
  font-size: 0.75rem;
  color: var(--color-text-muted);
}
.add-button {
  align-self: flex-start;
}
.error {
  color: var(--error-color);
}
.panel-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}
</style>
