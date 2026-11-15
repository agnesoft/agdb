import { computed, type ComputedRef, type Ref } from "vue";
import { buildQueryFromSteps } from "../utils/queryExecutionUtils";
import type { QueryRow, QueryStep, TAB } from "./types";
import { useQueryStore } from "./queryStore";
import { executeDbQueries, useMinimalRoute } from "./dbExec";
import useModal from "@agdb-studio/common/src/composables/modal/modal";
import type { Content } from "@agdb-studio/common/src/composables/content/types";
import { EMPHASIZED_CLASSNAME } from "@agdb-studio/common/src/composables/content/utils";
import { formatSteps } from "../utils/queryRowClipboard";

/** Queries in the confirmation are code: monospace, and long values wrap. */
const CODE_STYLE = { "font-family": "monospace", "word-break": "break-all" };

export const useQueryExecution = (
  queryId: Ref<string> | undefined,
  tab: Ref<TAB> | undefined,
  rows: ComputedRef<QueryRow[]>,
  isRunning: ComputedRef<boolean>,
) => {
  const queryStore = useQueryStore();
  const route = useMinimalRoute();
  const { openModal } = useModal();

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

    const id = queryId.value;
    // Snapshot now: the builder may change while the request runs, or while
    // the confirmation is open, and what was shown must be what runs.
    const run = snapshot();
    if (
      run.tab === "exec_mut" &&
      queryStore.getQuery(id)?.value.unconfirmedLinkedMutation
    ) {
      confirmLinkedMutation(id, run);
      return;
    }
    await execute(id, run);
  };

  type RunSnapshot = {
    tab: TAB | undefined;
    /** Copies of the rows' steps, so later edits don't change the run. */
    rows: QueryStep[][];
    /** Ids of those rows, so results stay tied to the rows that produced them. */
    rowIds: string[];
  };

  const snapshot = (): RunSnapshot => ({
    tab: tab?.value,
    rows: activeRows.value.map((row) =>
      row.steps.map((step) => ({
        ...step,
        ...(step.args
          ? { args: step.args.map((entry) => entry.map((f) => ({ ...f }))) }
          : {}),
      })),
    ),
    rowIds: activeRows.value.map((row) => row.id),
  });

  /** Asks before running exec_mut rows that came from a link, showing them in full. */
  const confirmLinkedMutation = (id: string, run: RunSnapshot) => {
    const code: Content[] = run.rows.map((steps) => ({
      paragraph: [{ text: formatSteps(steps), style: CODE_STYLE }],
    }));
    openModal({
      header: "Run queries from a link?",
      content: [
        {
          paragraph: [
            { text: "These queries were opened from a link and will " },
            { text: "change the database", className: EMPHASIZED_CLASSNAME },
            { text: ". Check them before running:" },
          ],
        },
        ...code,
      ],
      wide: true,
      onConfirm: () => {
        queryStore.setUnconfirmedLinkedMutation(id, false);
        void execute(id, run);
        return true;
      },
    });
  };

  const execute = async (id: string, run: RunSnapshot): Promise<void> => {
    const { tab: runTab, rows, rowIds } = run;
    try {
      await queryStore.runQuery(id, async () => {
        // All rows are sent in one request, so the server runs them as a single transaction.
        const queries = rows.map((steps) => buildQueryFromSteps(steps));
        const stepTypes = rows.flatMap((steps) =>
          steps.map((step) => step.type),
        );
        const results = await executeDbQueries(
          queries,
          runTab === "exec_mut",
          route,
        );
        if (results && runTab) {
          queryStore.setQueryResults(id, runTab, results, {
            queries,
            stepTypes,
            rowIds,
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
