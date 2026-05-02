<script lang="ts" setup>
import { computed, inject, onMounted, type Ref } from "vue";
import {
  OPTION_TYPE_MAP,
  VALUE_TYPES,
  type QueryArguments,
} from "../../../mock/queryApiMock";
import QueryArgumentDropdown from "./QueryArgumentDropdown.vue";
import type {
  QueryStep,
  QueryStepArgEntry,
  QueryStepFieldValue,
  TAB,
} from "../../../composables/types";
import { useQueryStore } from "../../../composables/queryStore";

const props = defineProps<{
  arguments: QueryArguments;
  step: QueryStep;
}>();

const queryId = inject<Ref<string>>("queryId");
const tab = inject<Ref<TAB>>("activeTab");
const queryStore = useQueryStore();

const VALUE_TYPE_SHORTCUTS: Record<string, string> = {
  string: "s",
  unsigned: "u",
  signed: "i",
  boolean: "b",
  float: "f",
  "string[]": "s[]",
  "unsigned[]": "u[]",
  "signed[]": "i[]",
  "boolean[]": "b[]",
  "float[]": "f[]",
};

const valueTypeSet = new Set<string>(VALUE_TYPES);

const isValueTypeField = (field: QueryArguments["fields"][number]) =>
  field.options.every((option) => valueTypeSet.has(option));

const makeEmptyEntry = (): QueryStepArgEntry =>
  props.arguments.fields.map((field) => ({
    selectedOption: field.options[0] ?? "",
    value: undefined,
  }));

const entries = computed((): QueryStepArgEntry[] =>
  props.step.args?.length ? props.step.args : [makeEmptyEntry()],
);

const commitArgs = (args: QueryStepArgEntry[]) => {
  if (!queryId?.value || !tab?.value) return;
  queryStore.updateQueryStep(queryId.value, tab.value, { ...props.step, args });
};

onMounted(() => {
  if (!props.step.args?.length) {
    commitArgs([makeEmptyEntry()]);
  }
});

const updateField = (
  entryIndex: number,
  fieldIndex: number,
  patch: Partial<QueryStepFieldValue>,
) => {
  const updated = entries.value.map((entry, ei) =>
    ei === entryIndex
      ? entry.map((fv, fi) => (fi === fieldIndex ? { ...fv, ...patch } : fv))
      : entry,
  );
  commitArgs(updated);
};

const addEntry = () => commitArgs([...entries.value, makeEmptyEntry()]);

const removeEntry = (index: number) =>
  commitArgs(entries.value.filter((_, i) => i !== index));
</script>

<template>
  <div class="query-argument">
    <div
      v-for="(entry, entryIndex) in entries"
      :key="entryIndex"
      class="arg-entry"
    >
      <div
        v-for="(field, fieldIndex) in props.arguments.fields"
        :key="fieldIndex"
        class="arg-field"
      >
        <QueryArgumentDropdown
          :options="field.options"
          :model-value="entry[fieldIndex]!.selectedOption"
          :is-value-type-field="isValueTypeField(field)"
          :shortcuts="VALUE_TYPE_SHORTCUTS"
          @update:model-value="
            updateField(entryIndex, fieldIndex, {
              selectedOption: $event,
              value: undefined,
            })
          "
        />
        <input
          v-if="
            OPTION_TYPE_MAP[entry[fieldIndex]!.selectedOption ?? ''] != null
          "
          class="arg-input"
          :value="entry[fieldIndex]!.value ?? ''"
          placeholder="value"
          @input="
            updateField(entryIndex, fieldIndex, {
              value: ($event.target as HTMLInputElement).value,
            })
          "
        />
      </div>
      <button
        v-if="props.arguments.repeatable && entries.length > 1"
        type="button"
        class="button button-danger arg-remove-entry"
        title="Remove entry"
        @click="removeEntry(entryIndex)"
      >
        −
      </button>
    </div>
    <button
      v-if="props.arguments.repeatable"
      type="button"
      class="button button-secondary arg-add-entry"
      title="Add entry"
      @click="addEntry"
    >
      +
    </button>
  </div>
</template>

<style lang="less" scoped>
.query-argument {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.25rem 0.5rem;
}

.arg-entry {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.arg-field {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.arg-input {
  font-size: 0.75rem;
  padding: 0.15rem 0.3rem;
  border: 1px solid var(--color-border);
  border-radius: 0.2rem;
  background-color: var(--color-background);
  color: var(--color-text);
}

.arg-input {
  width: 7rem;
}

.arg-add-entry,
.arg-remove-entry {
  padding: 0.1rem 0.4rem;
  font-size: 0.8rem;
  line-height: 1;
}
</style>
