import type { Components } from "@agnesoft/agdb_api";
import { ref, type Ref } from "vue";
import type { AddQueryParams, Query, QueryRow, QueryStep, TAB } from "./types";
import { queryApiMock } from "../mock/queryApiMock";

const queries = ref(new Map<string, Ref<Query>>());

const getQuery = (queryKey: string): Ref<Query> | undefined => {
  return queries.value.get(queryKey);
};

const createRow = (): QueryRow => ({
  id: `row-${crypto.randomUUID()}`,
  steps: [],
});

const addQuery = (query: AddQueryParams): Ref<Query> => {
  const newQuery: Ref<Query> = ref({
    ...query,
    isRunning: false,
    rows: {
      exec: [createRow()],
      exec_mut: [createRow()],
      context: [createRow()],
    },
    results: {},
  });
  queries.value.set(query.id, newQuery);
  return newQuery;
};

const deleteQuery = (queryId: string) => {
  queries.value.delete(queryId);
};

const findRow = (
  query: Ref<Query>,
  tab: TAB,
  rowId: string,
): QueryRow | undefined => {
  return query.value.rows[tab].find((r) => r.id === rowId);
};

/** Finds the row that contains the given step (step ids are unique across rows). */
const findRowByStepId = (
  query: Ref<Query>,
  tab: TAB,
  stepId: string,
): QueryRow | undefined => {
  return query.value.rows[tab].find((r) =>
    r.steps.some((s) => s.id === stepId),
  );
};

const addQueryRow = (queryId: string, tab: TAB): QueryRow | undefined => {
  const query = getQuery(queryId);
  if (!query?.value) return undefined;
  const row = createRow();
  query.value.rows[tab].push(row);
  return row;
};

/** Removes a row. The tab always keeps at least one (empty) row. */
const deleteQueryRow = (queryId: string, tab: TAB, rowId: string) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const remaining = query.value.rows[tab].filter((r) => r.id !== rowId);
  query.value.rows[tab] = remaining.length ? remaining : [createRow()];
};

/** Moves a row to a new position. Out-of-range indexes are ignored. */
const moveQueryRow = (
  queryId: string,
  tab: TAB,
  fromIndex: number,
  toIndex: number,
) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const rows = query.value.rows[tab];
  if (
    fromIndex === toIndex ||
    fromIndex < 0 ||
    toIndex < 0 ||
    fromIndex >= rows.length ||
    toIndex >= rows.length
  ) {
    return;
  }
  const [row] = rows.splice(fromIndex, 1);
  /* v8 ignore else -- @preserve: fromIndex is range-checked above */
  if (row) rows.splice(toIndex, 0, row);
};

const addQueryStep = (
  queryId: string,
  tab: TAB,
  rowId: string,
  step: QueryStep,
) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const row = findRow(query, tab, rowId);
  if (row) {
    row.steps.push(step);
    validateQuerySteps(queryId, tab);
  }
};

const updateQueryStep = (queryId: string, tab: TAB, step: QueryStep) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const row = findRowByStepId(query, tab, step.id);
  if (row) {
    const index = row.steps.findIndex((s) => s.id === step.id);
    row.steps[index] = step;
    validateQuerySteps(queryId, tab);
  }
};

const deleteQueryStep = (queryId: string, tab: TAB, stepId: string) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const row = findRowByStepId(query, tab, stepId);
  if (row) {
    row.steps = row.steps.filter((s) => s.id !== stepId);
    validateQuerySteps(queryId, tab);
  }
};

const clearQuerySteps = (queryId: string, tab: TAB, rowId: string) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const row = findRow(query, tab, rowId);
  if (row) {
    row.steps = [];
  }
};

const runQuery = async (
  queryId: string,
  runner?: () => Promise<void>,
): Promise<void> => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.isRunning = true;

    if (!runner) {
      // Simulate query execution when no real runner is provided.
      setTimeout(() => {
        query.value.isRunning = false;
        query.value.lastRun = new Date();
      }, 1000);
      return;
    }

    try {
      await runner();
    } finally {
      query.value.isRunning = false;
      query.value.lastRun = new Date();
    }
  }
};

const stopQuery = (queryId: string) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.isRunning = false;
  }
};

const setQueryResults = (
  queryId: string,
  tab: TAB,
  results: Components.Schemas.QueryResult[],
) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.results = {
      ...query.value.results,
      [tab]: results,
    };
    if (tab === "exec" || tab === "exec_mut") {
      query.value.lastExecutedTab = tab;
    }
  }
};

/** Clears stored results. Intended for use when query steps are modified or the view is reset. */
const clearQueryResults = (queryId: string, tab?: TAB) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  if (tab) {
    const updated = { ...query.value.results };
    delete updated[tab];
    query.value.results = updated;
  } else {
    query.value.results = {};
    query.value.lastExecutedTab = undefined;
  }
};

const clearQueries = () => {
  queries.value.clear();
};

const validateQuerySteps = (queryId: string, tab: TAB) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  for (const row of query.value.rows[tab]) {
    let followers: string[] =
      /* v8 ignore next */
      queryApiMock[tab === "exec_mut" ? "exec_mut" : "exec"]?.followers ?? [];

    for (const step of row.steps) {
      if (!followers.includes(step.type)) {
        step.invalid = true;
        followers = [];
      } else {
        step.invalid = false;
        /* v8 ignore next */
        followers = queryApiMock[step.type]?.followers ?? [];
      }
    }
  }
};

export const useQueryStore = () => {
  return {
    addQuery,
    deleteQuery,
    addQueryRow,
    deleteQueryRow,
    moveQueryRow,
    addQueryStep,
    updateQueryStep,
    deleteQueryStep,
    runQuery,
    stopQuery,
    clearQueries,
    getQuery,
    clearQuerySteps,
    validateQuerySteps,
    setQueryResults,
    clearQueryResults,
  };
};
