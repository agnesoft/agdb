<script lang="ts" setup>
import { computed, ref, toRef, useTemplateRef, watch } from "vue";
import { onClickOutside, useIntersectionObserver } from "@vueuse/core";
import {
  AkChevronDownSmall,
  AkChevronRightSmall,
  AkChevronUpSmall,
  AkMoreHorizontalFill,
  ClCloseMd,
  LuMaximize2,
  LuMinimize2,
  LuPencil,
  MdRefresh,
} from "@kalimahapps/vue-icons";
import SpinnerIcon from "@agdb-studio/design/src/components/icons/SpinnerIcon.vue";
import QueryResults from "@agdb-studio/query/src/components/results/QueryResults.vue";
import { useWidgetQuery } from "../composables/widgetQuery";
import { adminMode } from "../composables/dashboardStore";
import type { Widget, WidgetSize } from "../types";
import WidgetSizePicker from "./WidgetSizePicker.vue";

const props = defineProps<{
  widget: Widget;
  /** Its size on the grid, from the dashboard layout. */
  size: WidgetSize;
  first?: boolean;
  last?: boolean;
  /** Changes to re-run the queries (dashboard "Refresh all"). */
  refreshKey?: number;
  /** Shown without the actions that change the dashboard (read-only role). */
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  edit: [];
  remove: [];
  move: [delta: number];
  resize: [size: WidgetSize];
  /** Saves the widget with changed content (the colours and view shown). */
  update: [widget: Widget];
}>();

const {
  error,
  loading,
  revision,
  run,
  query,
  displayChanged,
  resetDisplay,
  shownWidget,
} = useWidgetQuery(toRef(props, "widget"));

const menuOpen = ref(false);
const menu = useTemplateRef<HTMLElement>("menu");
onClickOutside(menu, () => (menuOpen.value = false));

const menuToggle = useTemplateRef<HTMLButtonElement>("menuToggle");
/** Closes the menu from the keyboard; focus goes back to its button. */
const closeMenu = () => {
  menuOpen.value = false;
  menuToggle.value?.focus();
};

/** Runs a menu entry that leaves the widget's place (edit, remove) and closes the menu. */
const fromMenu = (event: "edit" | "remove") => {
  menuOpen.value = false;
  if (event === "edit") emit("edit");
  else emit("remove");
};

watch(
  () => props.refreshKey,
  () => void run(),
);

// The admin view opens the query editor with admin access, as the widget runs.
const queryRoute = adminMode() ? "admin-query" : "query";
const queryLink = computed(() => ({
  name: queryRoute,
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
      <h3 class="title" :title="widget.title || 'Untitled'">
        {{ widget.title || "Untitled" }}
      </h3>
      <span class="db" :title="`${widget.owner}/${widget.db}`">
        {{ widget.owner }}/{{ widget.db }}
      </span>
      <div
        v-if="displayChanged"
        class="view-note"
        data-testid="widget-view-changed"
      >
        <span
          title="The view, colours or pinned details changed here are not saved to the dashboard"
        >
          Unsaved view
        </span>
        <button
          v-if="!readOnly"
          type="button"
          data-testid="widget-view-save"
          @click="emit('update', shownWidget())"
        >
          Save
        </button>
        <button
          type="button"
          data-testid="widget-view-reset"
          @click="resetDisplay"
        >
          Reset
        </button>
      </div>
      <div class="actions">
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
        <div
          v-if="!readOnly"
          ref="menu"
          class="menu"
          @keydown.esc.stop="closeMenu"
        >
          <button
            ref="menuToggle"
            type="button"
            title="Widget options"
            aria-label="Widget options"
            :aria-expanded="menuOpen"
            data-testid="widget-menu"
            @click="menuOpen = !menuOpen"
          >
            <AkMoreHorizontalFill />
          </button>
          <div v-if="menuOpen" class="menu-items">
            <button
              type="button"
              data-testid="widget-edit"
              @click="fromMenu('edit')"
            >
              <LuPencil /> Edit widget…
            </button>
            <WidgetSizePicker :size="size" @resize="emit('resize', $event)" />
            <button
              type="button"
              data-testid="widget-move-up"
              :disabled="first"
              @click="emit('move', -1)"
            >
              <AkChevronUpSmall /> Move earlier
            </button>
            <button
              type="button"
              data-testid="widget-move-down"
              :disabled="last"
              @click="emit('move', 1)"
            >
              <AkChevronDownSmall /> Move later
            </button>
            <button
              type="button"
              class="danger"
              data-testid="widget-remove"
              @click="fromMenu('remove')"
            >
              <ClCloseMd /> Remove
            </button>
          </div>
        </div>
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
  display: flex;
  flex: none;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--color-border);
}
.title {
  min-width: 0;
  margin: 0;
  font-size: 1rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.db {
  // Pushes the note and actions to the right.
  margin-right: auto;
  flex-shrink: 1;
  min-width: 0;
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
.menu {
  position: relative;
}
.menu-items {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: var(--z-index-dropdown);
  display: flex;
  flex-direction: column;
  min-width: 11rem;
  padding: 0.25rem;
  border: 1px solid var(--color-border);
  border-radius: 0.375rem;
  background: var(--color-background);
  > button {
    justify-content: flex-start;
    gap: 0.5rem;
    width: 100%;
    padding: 0 0.5rem;
    font-size: 0.85rem;
    white-space: nowrap;
  }
  .danger {
    color: var(--error-color);
  }
}
.view-note {
  display: flex;
  flex: none;
  align-items: center;
  height: 1.5rem;
  padding-left: 0.5rem;
  border: 1px solid color-mix(in srgb, var(--warning-color) 45%, transparent);
  border-radius: 999px;
  background: color-mix(in srgb, var(--warning-color) 8%, transparent);
  font-size: 0.75rem;
  white-space: nowrap;
  color: var(--warning-color);
  span {
    padding-right: 0.4rem;
  }
  button {
    height: 100%;
    padding: 0 0.5rem;
    border: none;
    border-left: 1px solid
      color-mix(in srgb, var(--warning-color) 30%, transparent);
    background: transparent;
    color: var(--color-text);
    font-size: 0.75rem;
    cursor: pointer;
    &:last-child {
      border-radius: 0 999px 999px 0;
    }
    &:hover {
      background: var(--color-background-mute);
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
