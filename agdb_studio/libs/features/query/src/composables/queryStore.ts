import type { Components } from "@agnesoft/agdb_api";
import { ref, type Ref } from "vue";
import type {
  AddQueryParams,
  CopiedStep,
  Query,
  QueryResults,
  QueryRow,
  QueryStep,
  QueryType,
  TAB,
} from "./types";
import { queryApiMock } from "../mock/queryApiMock";

type DbElement = Components.Schemas.DbElement;

/** Every results bucket that can hold graph elements. */
const RESULT_KEYS = [
  "exec",
  "exec_mut",
  "context",
  "expanded",
] as const satisfies readonly (keyof QueryResults)[];

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

/**
 * Pastes copied steps (with new ids) into an empty row, or into a new row
 * right after it. Returns the row that received the steps.
 */
const pasteQuerySteps = (
  queryId: string,
  tab: TAB,
  rowId: string,
  steps: CopiedStep[],
): QueryRow | undefined => {
  const query = getQuery(queryId);
  if (!query?.value) return undefined;
  const rows = query.value.rows[tab];
  const index = rows.findIndex((r) => r.id === rowId);
  const current = rows[index];
  if (!current) return undefined;
  const pasted = steps.map(({ type, args }) => ({
    id: `step-${crypto.randomUUID()}`,
    type,
    // Each paste gets its own copy; plain copies also work for reactive args.
    ...(args
      ? { args: args.map((entry) => entry.map((field) => ({ ...field }))) }
      : {}),
  }));
  const targetIndex = current.steps.length ? index + 1 : index;
  if (targetIndex === index) {
    current.steps = pasted;
  } else {
    rows.splice(targetIndex, 0, { ...createRow(), steps: pasted });
  }
  validateQuerySteps(queryId, tab);
  // The row from the reactive array, so callers can change it reactively.
  return rows[targetIndex];
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

    const before = query.value.lastRun;
    try {
      await runner();
    } finally {
      query.value.isRunning = false;
      // Runs that stored results already set it (see setQueryResults).
      if (query.value.lastRun === before) query.value.lastRun = new Date();
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
  options: {
    keepExpanded?: boolean;
    /** The queries that produced `results`; kept so they can be re-run. */
    queries?: Components.Schemas.QueryType[];
    /** The step types of those queries, so later builder edits don't change the view. */
    stepTypes?: QueryType[];
  } = {},
) => {
  const query = getQuery(queryId);
  if (query?.value) {
    if (options.queries) {
      query.value.executedQueries = {
        ...query.value.executedQueries,
        [tab]: options.queries,
      };
    }
    if (options.stepTypes) {
      query.value.executedStepTypes = {
        ...query.value.executedStepTypes,
        [tab]: options.stepTypes,
      };
    }
    const updated: QueryResults = {
      ...query.value.results,
      [tab]: results,
    };
    if (tab === "exec" || tab === "exec_mut") {
      query.value.lastExecutedTab = tab;
      if (!options.keepExpanded) {
        // A freshly executed query starts with a clean graph. Its run time is
        // set together with the results, so watchers see both in one update.
        delete updated.expanded;
        query.value.lastRun = new Date();
      }
    }
    query.value.results = updated;
  }
};

/** Appends on-demand loaded elements (e.g. neighbours) to the graph. */
const appendExpandedResults = (
  queryId: string,
  results: Components.Schemas.QueryResult[],
) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.results = {
      ...query.value.results,
      expanded: [...(query.value.results.expanded ?? []), ...results],
    };
  }
};

const mapAllResults = (
  queryId: string,
  fn: (elements: DbElement[]) => DbElement[],
) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  const updated: QueryResults = {};
  for (const key of RESULT_KEYS) {
    const results = query.value.results[key];
    if (results) {
      updated[key] = results.map((r) => ({ ...r, elements: fn(r.elements) }));
    }
  }
  query.value.results = updated;
};

/**
 * Removes elements from all stored results. Edges connected to a removed
 * node are removed as well, mirroring the database behaviour.
 */
const removeElementsFromResults = (queryId: string, ids: number[]) => {
  const removed = new Set(ids);
  mapAllResults(queryId, (elements) =>
    elements.filter(
      (el) =>
        !removed.has(el.id) &&
        !(el.id < 0 && (removed.has(el.from) || removed.has(el.to))),
    ),
  );
};

/** Clears stored results. Intended for use when query steps are modified or the view is reset. */
const clearQueryResults = (queryId: string, tab?: TAB) => {
  const query = getQuery(queryId);
  if (!query?.value) return;
  if (tab) {
    const updated = { ...query.value.results };
    delete updated[tab];
    query.value.results = updated;
    const executed = { ...query.value.executedQueries };
    delete executed[tab];
    query.value.executedQueries = executed;
    const stepTypes = { ...query.value.executedStepTypes };
    delete stepTypes[tab];
    query.value.executedStepTypes = stepTypes;
  } else {
    query.value.results = {};
    query.value.lastExecutedTab = undefined;
    query.value.executedQueries = undefined;
    query.value.executedStepTypes = undefined;
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
    pasteQuerySteps,
    validateQuerySteps,
    setQueryResults,
    clearQueryResults,
    appendExpandedResults,
    removeElementsFromResults,
  };
};
