import { computed, ref } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { QueryStep, TAB } from "./types";

const {
  routeState,
  routeControl,
  runQuery,
  stopQuery,
  buildQueryFromSteps,
  checkClient,
  db_exec,
  db_exec_mut,
  admin_db_exec,
  admin_db_exec_mut,
} = vi.hoisted(() => ({
  routeState: {
    params: { owner: "alice", db: "main" },
    name: "db-view",
  } as { params: Record<string, unknown>; name?: unknown },
  routeControl: { throwUseRoute: false },
  runQuery: vi.fn(async (_queryId: string, runner?: () => Promise<void>) => {
    await runner?.();
  }),
  stopQuery: vi.fn(),
  buildQueryFromSteps: vi.fn(() => ({ built: true })),
  checkClient: vi.fn(),
  db_exec: vi.fn().mockResolvedValue({}),
  db_exec_mut: vi.fn().mockResolvedValue({}),
  admin_db_exec: vi.fn().mockResolvedValue({}),
  admin_db_exec_mut: vi.fn().mockResolvedValue({}),
}));

vi.mock("vue-router", () => ({
  useRoute: () => {
    if (routeControl.throwUseRoute) {
      throw new Error("no route available");
    }
    return routeState;
  },
}));

vi.mock("./queryStore", () => ({
  useQueryStore: () => ({
    runQuery,
    stopQuery,
  }),
}));

vi.mock("../utils/queryExecutionUtils", () => ({
  buildQueryFromSteps,
}));

vi.mock("@agdb-studio/api/src/api", () => ({
  checkClient,
  client: {
    value: {
      db_exec,
      db_exec_mut,
      admin_db_exec,
      admin_db_exec_mut,
    },
  },
}));

import { useQueryExecution } from "./queryExecution";

const makeStep = (type = "select"): QueryStep => ({
  id: type,
  type: type as QueryStep["type"],
  invalid: false,
});

const setup = ({
  queryId = "q1",
  tab = "exec" as TAB,
  steps = [makeStep()],
  isRunning = false,
}: {
  queryId?: string | undefined;
  tab?: TAB;
  steps?: QueryStep[];
  isRunning?: boolean;
} = {}) => {
  return useQueryExecution(
    ref(queryId),
    ref(tab),
    computed(() => steps),
    computed(() => isRunning),
  );
};

describe("useQueryExecution", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    routeState.params = { owner: "alice", db: "main" };
    routeState.name = "db-view";
    routeControl.throwUseRoute = false;
    runQuery.mockImplementation(
      async (_queryId: string, runner?: () => Promise<void>) => {
        await runner?.();
      },
    );
  });

  it("exposes run label and canRun for a runnable query", () => {
    const execution = setup();
    expect(execution.runLabel.value).toBe("Run query");
    expect(execution.canRun.value).toBe(true);
  });

  it("shows stop state while running", async () => {
    const execution = setup({ isRunning: true, steps: [] });
    expect(execution.runLabel.value).toBe("Stop query");
    expect(execution.canRun.value).toBe(false);

    await execution.runOrStopQuery();
    expect(stopQuery).toHaveBeenCalledWith("q1");
    expect(runQuery).not.toHaveBeenCalled();
  });

  it("does nothing when queryId is missing", async () => {
    const execution = setup({ queryId: "" });
    await execution.runOrStopQuery();
    expect(runQuery).not.toHaveBeenCalled();
    expect(stopQuery).not.toHaveBeenCalled();
  });

  it("calls db_exec for non-admin exec queries", async () => {
    const execution = setup({ tab: "exec" });
    await execution.runOrStopQuery();

    expect(checkClient).toHaveBeenCalled();
    expect(buildQueryFromSteps).toHaveBeenCalledWith([makeStep()]);
    expect(db_exec).toHaveBeenCalledWith({ owner: "alice", db: "main" }, [
      { built: true },
    ]);
  });

  it("calls db_exec_mut for mutation queries", async () => {
    const execution = setup({ tab: "exec_mut" });
    await execution.runOrStopQuery();

    expect(db_exec_mut).toHaveBeenCalledWith({ owner: "alice", db: "main" }, [
      { built: true },
    ]);
  });

  it("calls admin exec endpoints on admin routes", async () => {
    routeState.name = "admin-db-view";

    await setup({ tab: "exec" }).runOrStopQuery();
    expect(admin_db_exec).toHaveBeenCalledWith({ owner: "alice", db: "main" }, [
      { built: true },
    ]);

    vi.clearAllMocks();
    await setup({ tab: "exec_mut" }).runOrStopQuery();
    expect(admin_db_exec_mut).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [{ built: true }],
    );
  });

  it("skips API execution when route params are missing", async () => {
    routeState.params = { owner: "alice" };

    await setup().runOrStopQuery();

    expect(runQuery).toHaveBeenCalledWith("q1", expect.any(Function));
    expect(db_exec).not.toHaveBeenCalled();
    expect(admin_db_exec).not.toHaveBeenCalled();
  });

  it("falls back to an empty route when useRoute is unavailable", async () => {
    routeControl.throwUseRoute = true;

    await setup().runOrStopQuery();

    expect(runQuery).toHaveBeenCalledWith("q1", expect.any(Function));
    expect(db_exec).not.toHaveBeenCalled();
  });

  it("swallows execution failures", async () => {
    runQuery.mockRejectedValueOnce(new Error("boom"));

    await expect(setup().runOrStopQuery()).resolves.toBeUndefined();
  });
});
