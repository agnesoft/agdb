import { useDocumentVisibility, useIntervalFn } from "@vueuse/core";
import { computed, onScopeDispose, provide, ref, watch, type Ref } from "vue";
import {
  executeDbQueries,
  provideDbTarget,
} from "@agdb-studio/query/src/composables/dbExec";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { buildQueryFromSteps } from "@agdb-studio/query/src/utils/queryExecutionUtils";
import { WIDGET_TABS, type Widget } from "../types";
import { isRefreshInterval } from "../utils/refresh";
import { adminMode } from "./dashboardStore";
import {
  displayDiffers,
  seedDisplay,
  seedQuery,
  withDisplay,
} from "../utils/widgetRows";

const field = (value: unknown, key: string): unknown =>
  typeof value === "object" && value !== null && key in value
    ? Reflect.get(value, key)
    : undefined;

export const errorMessage = (error: unknown): string => {
  const response = field(error, "response");
  const status = field(response, "status");
  const data = field(response, "data");
  if (status === 403) return "No access to this database";
  if (status === 404) return "Database not found";
  if (typeof data === "string" && data) return data;
  return error instanceof Error ? error.message : String(error);
};

/**
 * The database a widget's queries run on; `admin` runs them with admin access
 * (the admin view runs every user's widgets so).
 */
export type WidgetTarget = { owner: string; db: string; admin?: boolean };

/**
 * Runs the exec and context rows of a query store entry on a database and
 * stores the results as the query view does. Results are stored only while
 * `isLatest()` holds, so a newer run wins over a slower older one. Gives
 * `false` when there is no exec row to run; throws when a row cannot be
 * built (e.g. saved unfinished) or the request fails. A `quiet` run shows
 * no global error notification; the caller shows the error.
 */
export const runEntryRows = async (
  queryId: string,
  target: WidgetTarget,
  isLatest: () => boolean,
  { quiet = false }: { quiet?: boolean } = {},
): Promise<boolean> => {
  const queryStore = useQueryStore();
  const query = queryStore.getQuery(queryId)?.value;
  /* v8 ignore next -- @preserve: callers run entries they keep */
  if (!query) return false;
  const tabs = WIDGET_TABS.map((tab) => {
    const rows = query.rows[tab].filter((row) => row.steps.length);
    return {
      tab,
      rows,
      queries: [] as ReturnType<typeof buildQueryFromSteps>[],
    };
  });
  if (!tabs[0]?.rows.length) return false;
  for (const t of tabs)
    t.queries = t.rows.map((row) => buildQueryFromSteps(row.steps));
  const route = {
    params: { owner: target.owner, db: target.db },
    name: target.admin ? "admin-query" : "query",
  };
  const results = await Promise.all(
    tabs.map((t) =>
      t.queries.length
        ? executeDbQueries(t.queries, false, route, { quiet })
        : Promise.resolve([]),
    ),
  );
  if (!isLatest()) return true;
  // Context first: the exec results mark the run (`lastRun`).
  for (const [i, t] of [...tabs.entries()].reverse()) {
    queryStore.setQueryResults(queryId, t.tab, results[i] ?? [], {
      queries: t.queries,
      stepTypes: t.rows.flatMap((row) => row.steps.map((s) => s.type)),
      rowIds: t.rows.map((row) => row.id),
    });
  }
  return true;
};

/**
 * Runs a widget's queries like the query view does: a query store entry holds
 * its rows, colours and view, and the query components below read it (they
 * get the entry id and the widget's database through provide/inject).
 * Re-runs when the widget changes and, with `refreshSeconds`, on an interval
 * that pauses while the browser tab is hidden.
 */
export const useWidgetQuery = (widget: Ref<Widget>) => {
  const queryStore = useQueryStore();
  const queryId = ref(`dashboard-${widget.value.id}-${crypto.randomUUID()}`);
  provide("queryId", queryId);
  // Captured once: the widget lives on one page (admin or not).
  const admin = adminMode();
  const target = (): WidgetTarget => ({
    owner: widget.value.owner,
    db: widget.value.db,
    admin,
  });
  provideDbTarget(target);

  const error = ref<string>();
  const loading = ref(false);
  /** Bumped when the entry is seeded again, so views reading it start over. */
  const revision = ref(0);
  let runId = 0;

  const visibility = useDocumentVisibility();
  // Other intervals would flood the server (too short, negative, or
  // overflowing the timer); they mean "off".
  const refreshSeconds = computed(() => {
    const seconds = widget.value.refreshSeconds ?? 0;
    return isRefreshInterval(seconds) ? seconds : 0;
  });
  const timer = useIntervalFn(
    () => {
      if (visibility.value !== "hidden") void run();
    },
    () => refreshSeconds.value * 1000,
    { immediate: false },
  );
  /** Resumes auto refresh; failed runs pause it until a run succeeds again. */
  const resumeRefresh = () => {
    if (refreshSeconds.value) timer.resume();
  };
  watch(
    refreshSeconds,
    (seconds) => (seconds ? timer.resume() : timer.pause()),
    { immediate: true },
  );

  const run = async (): Promise<void> => {
    const id = ++runId;
    loading.value = true;
    try {
      const ran = await runEntryRows(
        queryId.value,
        target(),
        () => id === runId,
      );
      // A newer run (e.g. after an edit) wins over a slower older one.
      if (id !== runId) return;
      if (!ran) {
        error.value = "No query yet: edit the widget to add one.";
        return;
      }
      error.value = undefined;
      resumeRefresh();
    } catch (e) {
      if (id !== runId) return;
      error.value = errorMessage(e);
      // Every failed request also shows a notification; retrying on the
      // interval would repeat it. The refresh button or an edit retries.
      timer.pause();
    } finally {
      if (id === runId) loading.value = false;
    }
  };

  // What the queries depend on; a title change doesn't re-run them.
  const source = computed(() =>
    JSON.stringify({
      owner: widget.value.owner,
      db: widget.value.db,
      rows: widget.value.rows,
    }),
  );
  watch(
    source,
    () => {
      seedQuery(queryId.value, widget.value);
      revision.value++;
      void run();
    },
    { immediate: true },
  );
  // Colours, view, pins and the chart only change how the results show: no
  // re-run.
  const display = computed(() =>
    JSON.stringify({
      colors: widget.value.colors,
      view: widget.value.view,
      pins: widget.value.pins,
      chart: widget.value.chart,
    }),
  );
  /** Shows the results with the widget's own colours, view and chart again. */
  const resetDisplay = () => {
    seedDisplay(queryId.value, widget.value);
    revision.value++;
  };
  // Saving what is shown (the view pill, the editor) needs no re-seed, which
  // would remount the results and lay the graph out again.
  watch(display, () => {
    if (displayChanged.value) resetDisplay();
  });

  const query = computed(() => queryStore.getQuery(queryId.value)?.value);
  /** Colours, view, pinned details or the chart were changed in the results and are not on the widget. */
  const displayChanged = computed(
    () => !!query.value && displayDiffers(widget.value, query.value),
  );
  /** The widget with the colours, view, pins and chart shown in its results. */
  const shownWidget = (): Widget =>
    /* v8 ignore next -- @preserve: the entry lives as long as the widget */
    query.value ? withDisplay(widget.value, query.value) : widget.value;

  onScopeDispose(() => queryStore.deleteQuery(queryId.value));

  return {
    queryId,
    revision,
    error,
    loading,
    run,
    query,
    displayChanged,
    resetDisplay,
    shownWidget,
  };
};
