<script lang="ts" setup>
import type { Components } from "@agnesoft/agdb_api";
import { computed, ref, watch } from "vue";
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

const title = computed(() =>
  isNode(props.element)
    ? `Node ${props.element.id}`
    : `Edge ${props.element.id}`,
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
watch(dirty, (value) => emit("dirty", value));

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
        type="button"
        class="button close-button"
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
      <table v-if="props.element.rawValues.length" class="values-table">
        <thead>
          <tr>
            <th>Key</th>
            <th>Value</th>
            <th>Type</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(kv, index) in props.element.rawValues"
            :key="index"
            data-testid="panel-value-row"
          >
            <td class="key">{{ dbValueToString(kv.key) }}</td>
            <td class="value">{{ dbValueToString(kv.value) }}</td>
            <td class="type">{{ dbValueTypeLabel(kv.value) }}</td>
          </tr>
        </tbody>
      </table>
      <p v-else class="empty">No values.</p>
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
      <div class="panel-actions">
        <button
          type="button"
          class="button button-primary"
          data-testid="panel-edit"
          :disabled="!valuesReady"
          :title="valuesReady ? undefined : 'Waiting for the stored values'"
          @click="emit('edit')"
        >
          Edit values
        </button>
      </div>
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
  justify-content: space-between;
  h3 {
    font-weight: bold;
  }
}
.close-button,
.remove-button {
  padding: 0 0.5rem;
  line-height: 1.5rem;
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
  margin-top: auto;
}
</style>
