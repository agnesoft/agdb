<script lang="ts" setup>
import { computed } from "vue";
import type { ColorSource, LegendEntry } from "../../utils/graphColorRules";

/**
 * What the graph's colours mean, so they never carry meaning on their own.
 * Grouped by where a colour comes from, as the same colour can mean a
 * property value in one group and a query in another.
 */
const props = defineProps<{
  entries: readonly LegendEntry[];
  palette: readonly string[];
}>();

const emit = defineEmits<{ edit: [] }>();

const GROUPS: { source: ColorSource; title: string }[] = [
  { source: "element", title: "Set by hand" },
  { source: "property", title: "By property" },
  { source: "query", title: "By query" },
];

const groups = computed(() =>
  GROUPS.map((group) => ({
    ...group,
    entries: props.entries.filter((entry) => entry.source === group.source),
  })).filter((group) => group.entries.length),
);
</script>

<template>
  <div class="color-legend" data-testid="color-legend">
    <section
      v-for="group in groups"
      :key="group.source"
      class="legend-group"
      data-testid="color-legend-group"
    >
      <h6 class="legend-title">{{ group.title }}</h6>
      <ul class="legend-entries" :aria-label="`Colours ${group.title}`">
        <li
          v-for="(entry, index) in group.entries"
          :key="index"
          class="legend-entry"
          data-testid="color-legend-entry"
        >
          <span
            class="legend-swatch"
            :style="{ backgroundColor: palette[entry.slot] }"
            aria-hidden="true"
          ></span>
          <span class="legend-label">{{ entry.label }}</span>
          <span class="legend-count" :title="`Colours ${entry.count}`">
            {{ entry.count }}
          </span>
        </li>
      </ul>
    </section>
    <button
      type="button"
      class="button button-transparent legend-edit"
      data-testid="color-legend-edit"
      @click="emit('edit')"
    >
      Edit colours
    </button>
  </div>
</template>

<style lang="less" scoped>
.color-legend {
  position: absolute;
  left: 0.5rem;
  bottom: 0.5rem;
  z-index: 1;
  max-width: 16rem;
  max-height: 40%;
  overflow-y: auto;
  margin: 0;
  padding: 0.35rem 0.5rem;
  font-size: 0.75rem;
  color: var(--color-text);
  background-color: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 0.3rem;
  opacity: 0.92;
}
.legend-group + .legend-group {
  margin-top: 0.25rem;
}
.legend-title {
  font-size: 0.7rem;
  font-weight: bold;
  color: var(--color-text-muted);
}
.legend-count {
  margin-left: auto;
  padding-left: 0.5rem;
  color: var(--color-text-muted);
}
.legend-entries {
  margin: 0;
  padding: 0;
  list-style: none;
}
.legend-edit {
  margin-top: 0.2rem;
  padding: 0;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  &:hover {
    color: var(--color-text);
  }
}
.legend-entry {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.legend-swatch {
  flex: 0 0 auto;
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 50%;
}
.legend-label {
  word-break: break-word;
}
</style>
