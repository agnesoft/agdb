<script lang="ts" setup>
import { computed, ref, toRef, useTemplateRef, watch } from "vue";
import { useIntersectionObserver } from "@vueuse/core";
import {
  AkChevronDownSmall,
  AkChevronRightSmall,
  AkChevronUpSmall,
  ClCloseMd,
  LuMaximize2,
  LuMinimize2,
  LuPencil,
  MdRefresh,
} from "@kalimahapps/vue-icons";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import QueryResults from "@agdb-studio/query/src/components/results/QueryResults.vue";
import { useWidgetQuery } from "../composables/widgetQuery";
import type { Widget, WidgetSize } from "../types";
import WidgetSizePicker from "./WidgetSizePicker.vue";

const props = defineProps<{
  widget: Widget;
  /** Its size on the grid, from the dashboard layout. */
  size: WidgetSize;
  editing?: boolean;
  first?: boolean;
  last?: boolean;
  /** Changes to re-run the queries (dashboard "Refresh all"). */
  refreshKey?: number;
}>();

const emit = defineEmits<{
  edit: [];
  remove: [];
  move: [delta: number];
  resize: [size: WidgetSize];
}>();

const { error, loading, revision, run, query } = useWidgetQuery(
  toRef(props, "widget"),
);

watch(
  () => props.refreshKey,
  () => void run(),
);

const queryLink = computed(() => ({
  name: "query",
  params: { owner: props.widget.owner, db: props.widget.db },
  query: {
    exec: props.widget.rows.exec.map((row) => row.query),
    context: props.widget.rows.context.map((row) => row.query),
  },
}));

const maximized = ref(false);

const gridStyle = computed(() => ({
  gridColumn: `span ${props.size.cols}`,
  gridRow: `span ${props.size.rows}`,
}));

// The results (a WebGL graph among them) are mounted only while on screen.
const body = useTemplateRef<HTMLElement>("body");
const visible = ref(false);
useIntersectionObserver(body, ([entry]) => {
  visible.value = !!entry?.isIntersecting;
});
const showResults = computed(() => visible.value || maximized.value);
const firstLoad = computed(() => loading.value && !query.value?.lastRun);
</script>

<template>
  <section
    class="dashboard-widget"
    :class="{ maximized }"
    :style="maximized ? undefined : gridStyle"
    :data-testid="`widget-${widget.id}`"
  >
    <header>
      <h3 class="title" :title="widget.title">{{ widget.title }}</h3>
      <span class="db" :title="`${widget.owner}/${widget.db}`">
        {{ widget.owner }}/{{ widget.db }}
      </span>
      <div class="actions">
        <template v-if="editing">
          <WidgetSizePicker :size="size" @resize="emit('resize', $event)" />
          <button
            type="button"
            title="Move earlier"
            aria-label="Move earlier"
            data-testid="widget-move-up"
            :disabled="first"
            @click="emit('move', -1)"
          >
            <AkChevronUpSmall />
          </button>
          <button
            type="button"
            title="Move later"
            aria-label="Move later"
            data-testid="widget-move-down"
            :disabled="last"
            @click="emit('move', 1)"
          >
            <AkChevronDownSmall />
          </button>
          <button
            type="button"
            title="Edit widget"
            aria-label="Edit widget"
            data-testid="widget-edit"
            @click="emit('edit')"
          >
            <LuPencil />
          </button>
          <button
            type="button"
            title="Remove widget"
            aria-label="Remove widget"
            data-testid="widget-remove"
            @click="emit('remove')"
          >
            <ClCloseMd />
          </button>
        </template>
        <template v-else>
          <button
            type="button"
            title="Refresh"
            aria-label="Refresh"
            data-testid="widget-refresh"
            @click="run"
          >
            <MdRefresh />
          </button>
          <RouterLink
            :to="queryLink"
            title="Open in query editor"
            aria-label="Open in query editor"
            data-testid="widget-open-query"
          >
            <AkChevronRightSmall />
          </RouterLink>
          <button
            type="button"
            :title="maximized ? 'Restore' : 'Maximize'"
            :aria-label="maximized ? 'Restore' : 'Maximize'"
            :aria-pressed="maximized"
            data-testid="widget-maximize"
            @click="maximized = !maximized"
          >
            <LuMinimize2 v-if="maximized" />
            <LuMaximize2 v-else />
          </button>
        </template>
      </div>
    </header>
    <div ref="body" class="body">
      <p v-if="error" class="error" data-testid="widget-error">{{ error }}</p>
      <SpinnerIcon v-else-if="firstLoad" />
      <QueryResults v-else-if="showResults" :key="revision" keep-view />
    </div>
  </section>
</template>

<style lang="less" scoped>
.dashboard-widget {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  border: 1px solid var(--color-border);
  border-radius: 0.5rem;
  background: var(--color-background-soft);
  &.maximized {
    position: fixed;
    inset: 1rem;
    z-index: var(--z-index-dropdown);
    border-color: var(--color-border-hover);
  }
}
header {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  flex: none;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--color-border);
}
.title {
  margin: 0;
  font-size: 1rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.db {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  max-width: 12rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.actions {
  display: flex;
  gap: 0.25rem;
  button,
  a {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--color-text);
    cursor: pointer;
    border-radius: 0.25rem;
    &:hover:not(:disabled) {
      background: var(--color-background-mute);
    }
    &:disabled {
      opacity: 0.4;
      cursor: default;
    }
  }
}
.body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 0.5rem;
  // Clips the results to the frame; the header stays open for the size popover.
  overflow: hidden;
  border-radius: 0 0 0.5rem 0.5rem;
  > * {
    flex: 1;
    min-height: 0;
  }
}
.error {
  color: var(--error-color);
  margin: auto;
  text-align: center;
}
</style>
