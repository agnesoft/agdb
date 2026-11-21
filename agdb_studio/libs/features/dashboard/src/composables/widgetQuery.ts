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
import { seedDisplay, seedQuery } from "../utils/widgetRows";

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
  provideDbTarget(() => ({ owner: widget.value.owner, db: widget.value.db }));

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
    const query = queryStore.getQuery(queryId.value)?.value;
    /* v8 ignore next -- @preserve: the entry lives as long as the widget */
    if (!query) return;
    const filled = WIDGET_TABS.map((tab) => ({
      tab,
      rows: query.rows[tab].filter((row) => row.steps.length),
    }));
    if (!filled[0]?.rows.length) {
      error.value = "No query yet: edit the widget to add one.";
      loading.value = false;
      return;
    }
    loading.value = true;
    const route = {
      params: { owner: widget.value.owner, db: widget.value.db },
    };
    try {
      // An unfinished chain (e.g. saved before it was complete) fails here.
      const tabs = filled.map((t) => ({
        ...t,
        queries: t.rows.map((row) => buildQueryFromSteps(row.steps)),
      }));
      const results = await Promise.all(
        tabs.map((t) =>
          t.queries.length
            ? executeDbQueries(t.queries, false, route)
            : Promise.resolve([]),
        ),
      );
      // A newer run (e.g. after an edit) wins over a slower older one.
      if (id !== runId) return;
      // Context first: the exec results mark the run (`lastRun`).
      const withResults = tabs.map((t, i) => ({ ...t, results: results[i] }));
      for (const t of withResults.reverse()) {
        queryStore.setQueryResults(queryId.value, t.tab, t.results ?? [], {
          queries: t.queries,
          stepTypes: t.rows.flatMap((row) => row.steps.map((s) => s.type)),
          rowIds: t.rows.map((row) => row.id),
        });
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
  // Colours and view only change how the results show: no re-run.
  const display = computed(() =>
    JSON.stringify({ colors: widget.value.colors, view: widget.value.view }),
  );
  watch(display, () => {
    seedDisplay(queryId.value, widget.value);
    revision.value++;
  });

  onScopeDispose(() => queryStore.deleteQuery(queryId.value));

  return {
    queryId,
    revision,
    error,
    loading,
    run,
    query: computed(() => queryStore.getQuery(queryId.value)?.value),
  };
};
