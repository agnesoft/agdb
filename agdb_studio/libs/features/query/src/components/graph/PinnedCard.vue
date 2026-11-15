<script lang="ts" setup>
import { computed } from "vue";
import { LuPinOff } from "@kalimahapps/vue-icons";
import type { GraphElement } from "../../composables/types";
import { isNode } from "../../composables/graphActions";
import { toDisplayValues } from "../../utils/graphDataUtils";
import { linkEndpointId } from "../../utils/graphStyle";

/**
 * Details of a pinned element that stay on the graph, next to the element.
 * Dragged by its header; the graph keeps it attached as the layout moves.
 */
const props = defineProps<{
  element: GraphElement;
  /** Top-left corner in px, relative to the graph. */
  x: number;
  y: number;
}>();

const emit = defineEmits<{
  unpin: [];
  "drag-start": [event: PointerEvent];
  open: [];
}>();

const title = computed(
  () => `${isNode(props.element) ? "Node" : "Edge"} ${props.element.id}`,
);

const endpoints = computed(() =>
  isNode(props.element)
    ? undefined
    : `${linkEndpointId(props.element.source)} → ${linkEndpointId(props.element.target)}`,
);

const queryOnly = computed(() =>
  toDisplayValues(props.element.queryOnlyValues ?? []),
);
</script>

<template>
  <div
    class="pinned-card"
    data-testid="pinned-card"
    :style="{ transform: `translate(${x}px, ${y}px)` }"
  >
    <header
      class="card-header"
      title="Drag to move, double-click to open the details"
      @pointerdown.left="emit('drag-start', $event)"
      @dblclick="emit('open')"
    >
      <span class="card-title">
        {{ title }}
        <span v-if="element.alias" class="card-alias">{{ element.alias }}</span>
      </span>
      <button
        type="button"
        class="button unpin-button"
        title="Unpin"
        aria-label="Unpin"
        data-testid="pinned-card-unpin"
        @pointerdown.stop
        @click="emit('unpin')"
      >
        <LuPinOff />
      </button>
    </header>
    <div v-if="endpoints" class="card-row card-endpoints">{{ endpoints }}</div>
    <div
      v-for="(v, index) in element.values"
      :key="`v${index}`"
      class="card-row"
    >
      <span class="card-key">{{ v.key }}</span> {{ v.value }}
    </div>
    <div v-for="(v, index) in queryOnly" :key="`q${index}`" class="card-row">
      <span class="card-key">{{ v.key }}</span> {{ v.value }}
      <span class="card-tag">query</span>
    </div>
    <div
      v-if="!element.values.length && !queryOnly.length"
      class="card-row card-empty"
    >
      {{ element.valuesState === "loading" ? "Loading…" : "No values" }}
    </div>
  </div>
</template>

<style lang="less" scoped>
.pinned-card {
  position: absolute;
  top: 0;
  left: 0;
  width: max-content;
  max-width: 16rem;
  max-height: 14rem;
  overflow-y: auto;
  padding: 0.3rem 0.5rem 0.4rem;
  font-size: 0.8rem;
  line-height: 1.3;
  color: var(--color-text);
  background-color: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 0.3rem;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.25);
  pointer-events: auto;
}
.card-header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-bottom: 0.15rem;
  cursor: grab;
  user-select: none;
  &:active {
    cursor: grabbing;
  }
}
.card-title {
  font-weight: bold;
  margin-right: auto;
}
.card-alias,
.card-key,
.card-endpoints,
.card-empty {
  font-weight: normal;
  color: var(--color-text-muted);
}
.card-row {
  word-break: break-word;
}
.card-tag {
  font-size: 0.7rem;
  padding: 0 0.3rem;
  border-radius: 0.3rem;
  border: 1px solid var(--color-border);
  color: var(--color-text-muted);
}
.unpin-button {
  display: flex;
  align-items: center;
  padding: 0.1rem;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  &:hover {
    color: var(--color-text);
  }
}
</style>
