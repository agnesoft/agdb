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
): Promise<void> => {
  checkClient(client);

  const ownerParam = route.params["owner"];
  const dbParam = route.params["db"];
  const owner = typeof ownerParam === "string" ? ownerParam : "";
  const db = typeof dbParam === "string" ? dbParam : "";
  if (!owner || !db) {
    return;
  }

  const payload = [buildQueryFromSteps(steps)];
  const isMutation = activeTab === "exec_mut";
  const useAdminEndpoints = isAdminRoute;

  if (useAdminEndpoints) {
    if (isMutation) {
      await client.value.admin_db_exec_mut({ owner, db }, payload);
    } else {
      await client.value.admin_db_exec({ owner, db }, payload);
    }
    return;
  }

  if (isMutation) {
    await client.value.db_exec_mut({ owner, db }, payload);
  } else {
    await client.value.db_exec({ owner, db }, payload);
  }
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
        await executeQuery(steps.value, tab?.value, route, isAdminRoute.value);
      });
    } catch {
      // Request failures are handled by API interceptors/notifications.
    }
  };

  return {
    runLabel,
    canRun,
    runOrStopQuery,
  };
};
