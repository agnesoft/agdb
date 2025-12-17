import { ref, type Ref } from "vue";
import type { AddQueryParams, Query, QueryStep } from "./types";

const queries = ref(new Map<string, Ref<Query>>());

const getQuery = (queryKey: string): Ref<Query> | undefined => {
  return queries.value.get(queryKey);
};

const addQuery = (query: AddQueryParams): Ref<Query> => {
  const newQuery: Ref<Query> = ref({
    ...query,
    isRunning: false,
  });
  queries.value.set(query.id, newQuery);
  return newQuery;
};

const deleteQuery = (queryId: string) => {
  queries.value.delete(queryId);
};

const addQueryStep = (queryId: string, step: QueryStep) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.steps.push(step);
  }
};

const updateQueryStep = (queryId: string, step: QueryStep) => {
  const query = getQuery(queryId);
  if (query?.value) {
    const index = query.value.steps.findIndex((s) => s.id === step.id);
    if (index !== -1) {
      query.value.steps[index] = step;
    }
  }
};

const deleteQueryStep = (queryId: string, stepId: string) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.steps = query.value.steps.filter((s) => s.id !== stepId);
  }
};

const runQuery = (queryId: string) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.isRunning = true;
    // Simulate query execution
    setTimeout(() => {
      query.value.isRunning = false;
      query.value.lastRun = new Date();
    }, 1000);
  }
};

const stopQuery = (queryId: string) => {
  const query = getQuery(queryId);
  if (query?.value) {
    query.value.isRunning = false;
  }
};

const clearQueries = () => {
  queries.value.clear();
};

export const useQueryStore = () => {
  return {
    addQuery,
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
