import type { Query, QueryStep } from "./types";

const queries = new Map<symbol, Query>();

const addQuery = (query: Query) => {
  queries.set(Symbol(query.id), query);
};

const updateQuery = (query: Query) => {
  const key = Array.from(queries.keys()).find(
    (k) => queries.get(k)?.id === query.id,
  );
  if (key) {
    queries.set(key, query);
  }
};

const deleteQuery = (queryId: string) => {
  const key = Array.from(queries.keys()).find(
    (k) => queries.get(k)?.id === queryId,
  );
  if (key) {
    queries.delete(key);
  }
};

const addQueryStep = (queryId: string, step: QueryStep) => {
  const query = Array.from(queries.values()).find((q) => q.id === queryId);
  if (query) {
    query.steps.push(step);
    updateQuery(query);
  }
};

const updateQueryStep = (queryId: string, step: QueryStep) => {
  const query = Array.from(queries.values()).find((q) => q.id === queryId);
  if (query) {
    const stepIndex = query.steps.findIndex((s) => s.id === step.id);
    if (stepIndex !== -1) {
      query.steps[stepIndex] = step;
      updateQuery(query);
    }
  }
};

const deleteQueryStep = (queryId: string, stepId: string) => {
  const query = Array.from(queries.values()).find((q) => q.id === queryId);
  if (query) {
    query.steps = query.steps.filter((s) => s.id !== stepId);
    updateQuery(query);
  }
};

const runQuery = (queryId: string) => {
  const query = Array.from(queries.values()).find((q) => q.id === queryId);
  if (query) {
    query.isRunning = true;
    updateQuery(query);
    // Simulate query execution
    setTimeout(() => {
      query.isRunning = false;
      query.lastRun = new Date();
      updateQuery(query);
    }, 1000);
  }
};

const stopQuery = (queryId: string) => {
  const query = Array.from(queries.values()).find((q) => q.id === queryId);
  if (query) {
    query.isRunning = false;
    updateQuery(query);
  }
};

const clearQueries = () => {
  queries.clear();
};

const getQuery = (queryId: string): Query | undefined => {
  return Array.from(queries.values()).find((q) => q.id === queryId);
};

export const useQueryStore = () => {
  return {
    addQuery,
    updateQuery,
    deleteQuery,
    addQueryStep,
    updateQueryStep,
    deleteQueryStep,
    runQuery,
    stopQuery,
    clearQueries,
    getQuery,
  };
};
