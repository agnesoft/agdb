import type { Components } from "@agnesoft/agdb_api";
import { computed, type ComputedRef, type Ref } from "vue";
import { useRoute } from "vue-router";
import { checkClient, client } from "@agdb-studio/api/src/api";
import { buildQueryFromSteps } from "../utils/queryExecutionUtils";
import type { QueryStep as QueryStepModel, TAB } from "./types";
import { useQueryStore } from "./queryStore";

type MinimalRoute = {
  params: Record<string, unknown>;
  name?: unknown;
};

const executeQuery = async (
  steps: QueryStepModel[],
  activeTab: TAB | undefined,
  route: MinimalRoute,
  isAdminRoute: boolean,
): Promise<Components.Schemas.QueryResult[] | undefined> => {
  checkClient(client);

  const ownerParam = route.params["owner"];
  const dbParam = route.params["db"];
  const owner = typeof ownerParam === "string" ? ownerParam : "";
  const db = typeof dbParam === "string" ? dbParam : "";
  if (!owner || !db) {
    return undefined;
  }

  const payload = [buildQueryFromSteps(steps)];
  const isMutation = activeTab === "exec_mut";
  const useAdminEndpoints = isAdminRoute;

  let response: { data: Components.Schemas.QueryResult[] };

  if (useAdminEndpoints) {
    if (isMutation) {
      response = await client.value.admin_db_exec_mut({ owner, db }, payload);
    } else {
      response = await client.value.admin_db_exec({ owner, db }, payload);
    }
  } else if (isMutation) {
    response = await client.value.db_exec_mut({ owner, db }, payload);
  } else {
    response = await client.value.db_exec({ owner, db }, payload);
  }

  return response.data;
};

export const useQueryExecution = (
  queryId: Ref<string> | undefined,
  tab: Ref<TAB> | undefined,
  steps: ComputedRef<QueryStepModel[]>,
  isRunning: ComputedRef<boolean>,
) => {
  const queryStore = useQueryStore();
  const route: MinimalRoute = (() => {
    try {
      return useRoute() as unknown as MinimalRoute;
    } catch {
      return { params: {}, name: undefined };
    }
  })();

  const isAdminRoute = computed(() => {
    return String(route.name ?? "").startsWith("admin-");
  });

  const runLabel = computed(() =>
    isRunning.value ? "Stop query" : "Run query",
  );

  const canRun = computed(() => {
    return (
      !!queryId?.value &&
      steps.value.length > 0 &&
      steps.value.every((step) => !step.invalid)
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
        const results = await executeQuery(
          steps.value,
          tab?.value,
          route,
          isAdminRoute.value,
        );
        if (results && queryId?.value && tab?.value) {
          queryStore.setQueryResults(queryId.value, tab.value, results);
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
