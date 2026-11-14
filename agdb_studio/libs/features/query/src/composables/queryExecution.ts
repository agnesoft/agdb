import { computed, type ComputedRef, type Ref } from "vue";
import { buildQueryFromSteps } from "../utils/queryExecutionUtils";
import type { QueryRow, TAB } from "./types";
import { useQueryStore } from "./queryStore";
import { executeDbQueries, useMinimalRoute } from "./dbExec";

export const useQueryExecution = (
  queryId: Ref<string> | undefined,
  tab: Ref<TAB> | undefined,
  rows: ComputedRef<QueryRow[]>,
  isRunning: ComputedRef<boolean>,
) => {
  const queryStore = useQueryStore();
  const route = useMinimalRoute();

  const runLabel = computed(() =>
    isRunning.value ? "Stop query" : "Run query",
  );

  // Empty rows are ignored so a freshly added row doesn't block running.
  const activeRows = computed(() =>
    rows.value.filter((row) => row.steps.length > 0),
  );

  const canRun = computed(() => {
    return (
      !!queryId?.value &&
      activeRows.value.length > 0 &&
      activeRows.value.every((row) => row.steps.every((step) => !step.invalid))
    );
  });

  const runOrStopQuery = async (): Promise<void> => {
    if (!queryId?.value) return;

    if (isRunning.value) {
      queryStore.stopQuery(queryId.value);
      return;
    }

    if (!canRun.value) return;

    try {
      await queryStore.runQuery(queryId.value, async () => {
        // All rows are sent in one request, so the server runs them as a single transaction.
        const queries = activeRows.value.map((row) =>
          buildQueryFromSteps(row.steps),
        );
        // Snapshot now: the builder may change while the request runs.
        const stepTypes = activeRows.value.flatMap((row) =>
          row.steps.map((step) => step.type),
        );
        const results = await executeDbQueries(
          queries,
          tab?.value === "exec_mut",
          route,
        );
        if (results && queryId?.value && tab?.value) {
          queryStore.setQueryResults(queryId.value, tab.value, results, {
            queries,
            stepTypes,
          });
        }
      });
    } catch (error) {
      // Auth errors (401) are handled by API interceptors.
      // Log other failures so they are visible during development.
      console.error("[query] execution failed:", error);
    }
  };

  return {
    runLabel,
    canRun,
    runOrStopQuery,
  };
};
