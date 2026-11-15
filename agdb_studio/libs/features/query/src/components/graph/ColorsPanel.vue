<script lang="ts" setup>
import type { ColorRule, PaletteSlot } from "../../utils/graphColorRules";
import ColorSwatchPicker from "./ColorSwatchPicker.vue";
import ColorRulesEditor, { type ValueIndexEntry } from "./ColorRulesEditor.vue";

/** One query of the last run, as it can be coloured. */
export type ColorQuery = {
  rowId: string;
  index: number;
  label: string;
  slot?: PaletteSlot;
};

/** An element coloured by hand. */
export type ColorElement = { id: number; slot: PaletteSlot; label: string };

/**
 * Everything that colours the graph, in the order it applies: by query, by
 * property (overrides queries), by hand on single elements (overrides all).
 */
defineProps<{
  palette: readonly string[];
  queries: readonly ColorQuery[];
  rules: readonly ColorRule[];
  keys: readonly ValueIndexEntry[];
  elements: readonly ColorElement[];
}>();

const emit = defineEmits<{
  "row-color": [rowId: string, slot: PaletteSlot | undefined];
  "each-query": [];
  "update:rules": [rules: ColorRule[]];
  "element-color": [id: number, slot: PaletteSlot | undefined];
  "clear-all": [];
  close: [];
}>();
</script>

<template>
  <div
    class="colors-panel"
    data-testid="colors-panel"
    role="dialog"
    aria-label="Colours"
    @keydown.esc.stop="emit('close')"
  >
    <header class="panel-header">
      <h4>Colours</h4>
      <button
        type="button"
        class="button close-button"
        aria-label="Close colours"
        @click="emit('close')"
      >
        ×
      </button>
    </header>

    <section class="section" data-testid="colors-queries">
      <h5>By query</h5>
      <p v-if="!queries.length" class="hint">
        Run a query to colour what each query returns.
      </p>
      <template v-else>
        <div
          v-for="q in queries"
          :key="q.rowId"
          class="query-line"
          data-testid="colors-query"
        >
          <span class="query-index">{{ q.index }}</span>
          <span class="query-label" :title="q.label">{{ q.label }}</span>
          <ColorSwatchPicker
            inline
            :palette="palette"
            :model-value="q.slot"
            :label="`Colour of query ${q.index}`"
            @update:model-value="(slot) => emit('row-color', q.rowId, slot)"
          />
        </div>
        <button
          v-if="queries.length > 1"
          type="button"
          class="button button-transparent link-button"
          data-testid="colors-each-query"
          @click="emit('each-query')"
        >
          Give each query its own colour
        </button>
      </template>
    </section>

    <section class="section" data-testid="colors-properties">
      <h5>By property <span class="overrides">overrides queries</span></h5>
      <ColorRulesEditor
        :rules="rules"
        :palette="palette"
        :keys="keys"
        @update:rules="(next) => emit('update:rules', next)"
      />
    </section>

    <section class="section" data-testid="colors-elements">
      <h5>Single elements <span class="overrides">override all</span></h5>
      <div
        v-for="el in elements"
        :key="el.id"
        class="query-line"
        data-testid="colors-element"
      >
        <span class="query-label">{{ el.label }}</span>
        <ColorSwatchPicker
          inline
          :palette="palette"
          :model-value="el.slot"
          :label="`Colour of ${el.label}`"
          @update:model-value="(slot) => emit('element-color', el.id, slot)"
        />
      </div>
      <p class="hint">
        Right-click a node or edge in the graph to give it its own colour.
      </p>
    </section>

    <button
      type="button"
      class="button button-transparent clear-button"
      data-testid="color-clear-all"
      @click="emit('clear-all')"
    >
      Clear all colours
    </button>
  </div>
</template>

<style lang="less" scoped>
.colors-panel {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  width: 26rem;
  max-width: calc(100vw - 2rem);
  max-height: 70vh;
  overflow-y: auto;
  padding: 0.6rem 0.75rem;
  font-size: 0.85rem;
  background-color: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 0.4rem;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  h4 {
    font-weight: bold;
  }
}
.close-button {
  padding: 0 0.4rem;
  line-height: 1.3rem;
}
.section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--color-border);
  h5 {
    font-weight: bold;
    font-size: 0.8rem;
  }
}
.overrides {
  margin-left: 0.3rem;
  font-weight: normal;
  color: var(--color-text-muted);
}
.query-line {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.query-index {
  flex: 0 0 1.2rem;
  text-align: right;
  color: var(--color-text-muted);
}
.query-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}
.link-button,
.clear-button {
  align-self: flex-start;
  padding: 0;
  font-size: 0.8rem;
}
.clear-button {
  opacity: 0.7;
  &:hover {
    opacity: 1;
    color: var(--error-color);
  }
}
</style>
