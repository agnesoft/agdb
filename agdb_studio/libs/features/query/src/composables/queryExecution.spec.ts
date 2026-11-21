import { computed, ref } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { QueryRow, QueryStep, TAB } from "./types";

const mockResults = [{ result: 1, elements: [] }];

const {
  routeState,
  routeControl,
  runQuery,
  stopQuery,
  setQueryResults,
  buildQueryFromSteps,
  checkClient,
  db_exec,
  db_exec_mut,
  admin_db_exec,
  admin_db_exec_mut,
  queryState,
  setUnconfirmedLinkedMutation,
  openModal,
} = vi.hoisted(() => ({
  queryState: { unconfirmedLinkedMutation: false as boolean | undefined },
  setUnconfirmedLinkedMutation: vi.fn(),
  openModal: vi.fn(),
  routeState: {
    params: { owner: "alice", db: "main" },
    name: "db-view",
  } as { params: Record<string, unknown>; name?: unknown },
  routeControl: { throwUseRoute: false },
  runQuery: vi.fn(async (_queryId: string, runner?: () => Promise<void>) => {
    await runner?.();
  }),
  stopQuery: vi.fn(),
  setQueryResults: vi.fn(),
  buildQueryFromSteps: vi.fn(() => ({ built: true })),
  checkClient: vi.fn(),
  db_exec: vi.fn().mockResolvedValue({ data: [{ result: 1, elements: [] }] }),
  db_exec_mut: vi
    .fn()
    .mockResolvedValue({ data: [{ result: 1, elements: [] }] }),
  admin_db_exec: vi
    .fn()
    .mockResolvedValue({ data: [{ result: 1, elements: [] }] }),
  admin_db_exec_mut: vi
    .fn()
    .mockResolvedValue({ data: [{ result: 1, elements: [] }] }),
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
    setQueryResults,
    setUnconfirmedLinkedMutation,
    getQuery: (id: string) => (id === "q1" ? { value: queryState } : undefined),
  }),
}));

vi.mock("@agdb-studio/common/src/composables/modal/modal", () => ({
  default: () => ({ openModal }),
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
  rows,
  isRunning = false,
}: {
  queryId?: string | undefined;
  tab?: TAB;
  steps?: QueryStep[];
  rows?: QueryRow[];
  isRunning?: boolean;
} = {}) => {
  const queryRows = rows ?? [{ id: "r1", steps }];
  return useQueryExecution(
    ref(queryId),
    ref(tab),
    computed(() => queryRows),
    computed(() => isRunning),
  );
};

describe("useQueryExecution", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    routeState.params = { owner: "alice", db: "main" };
    routeState.name = "db-view";
    routeControl.throwUseRoute = false;
    queryState.unconfirmedLinkedMutation = undefined;
    runQuery.mockImplementation(
      async (_queryId: string, runner?: () => Promise<void>) => {
        await runner?.();
      },
    );
    db_exec.mockResolvedValue({ data: mockResults });
    db_exec_mut.mockResolvedValue({ data: mockResults });
    admin_db_exec.mockResolvedValue({ data: mockResults });
    admin_db_exec_mut.mockResolvedValue({ data: mockResults });
  });

  it("exposes run label and canRun for a runnable query", () => {
    const execution = setup();
    expect(execution.runLabel.value).toBe("Run query");
    expect(execution.canRun.value).toBe(true);
  });

  it("disables invalid queries", () => {
    const execution = setup({
      steps: [{ ...makeStep(), invalid: true }],
    });

    expect(execution.canRun.value).toBe(false);
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
    expect(db_exec).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [{ built: true }],
      undefined,
    );
  });

  it("sends every non-empty row as one transaction payload", async () => {
    const execution = setup({
      rows: [
        { id: "r1", steps: [makeStep("select")] },
        { id: "r2", steps: [] },
        { id: "r3", steps: [makeStep("search")] },
      ],
    });

    expect(execution.canRun.value).toBe(true);
    await execution.runOrStopQuery();

    expect(buildQueryFromSteps).toHaveBeenCalledTimes(2);
    expect(buildQueryFromSteps).toHaveBeenNthCalledWith(1, [
      makeStep("select"),
    ]);
    expect(buildQueryFromSteps).toHaveBeenNthCalledWith(2, [
      makeStep("search"),
    ]);
    expect(db_exec).toHaveBeenCalledOnce();
    expect(db_exec).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [{ built: true }, { built: true }],
      undefined,
    );
  });

  it("cannot run when all rows are empty", () => {
    const execution = setup({
      rows: [
        { id: "r1", steps: [] },
        { id: "r2", steps: [] },
      ],
    });
    expect(execution.canRun.value).toBe(false);
  });

  it("cannot run when any row has an invalid step", () => {
    const execution = setup({
      rows: [
        { id: "r1", steps: [makeStep()] },
        { id: "r2", steps: [{ ...makeStep(), invalid: true }] },
      ],
    });
    expect(execution.canRun.value).toBe(false);
  });

  it("does not execute invalid queries", async () => {
    const execution = setup({
      steps: [{ ...makeStep(), invalid: true }],
    });

    await execution.runOrStopQuery();

    expect(runQuery).not.toHaveBeenCalled();
    expect(db_exec).not.toHaveBeenCalled();
    expect(db_exec_mut).not.toHaveBeenCalled();
  });

  it("calls db_exec_mut for mutation queries", async () => {
    const execution = setup({ tab: "exec_mut" });
    await execution.runOrStopQuery();

    expect(db_exec_mut).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [{ built: true }],
      undefined,
    );
  });

  it("calls admin exec endpoints on admin routes", async () => {
    routeState.name = "admin-db-view";

    await setup({ tab: "exec" }).runOrStopQuery();
    expect(admin_db_exec).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [{ built: true }],
      undefined,
    );

    vi.clearAllMocks();
    await setup({ tab: "exec_mut" }).runOrStopQuery();
    expect(admin_db_exec_mut).toHaveBeenCalledWith(
      { owner: "alice", db: "main" },
      [{ built: true }],
      undefined,
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

  it("swallows execution failures but logs them", async () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    runQuery.mockRejectedValueOnce(new Error("boom"));

    await expect(setup().runOrStopQuery()).resolves.toBeUndefined();
    expect(consoleSpy).toHaveBeenCalledWith(
      "[query] execution failed:",
      expect.any(Error),
    );

    consoleSpy.mockRestore();
  });

  it("stores query results after successful exec execution", async () => {
    const execution = setup({ tab: "exec" });
    await execution.runOrStopQuery();

    expect(setQueryResults).toHaveBeenCalledWith("q1", "exec", mockResults, {
      queries: [{ built: true }],
      stepTypes: ["select"],
      rowIds: ["r1"],
    });
  });

  it("stores query results after successful exec_mut execution", async () => {
    const execution = setup({ tab: "exec_mut" });
    await execution.runOrStopQuery();

    expect(setQueryResults).toHaveBeenCalledWith(
      "q1",
      "exec_mut",
      mockResults,
      {
        queries: [{ built: true }],
        stepTypes: ["select"],
        rowIds: ["r1"],
      },
    );
  });

  it("stores query results for context tab", async () => {
    const execution = setup({ tab: "context" });
    await execution.runOrStopQuery();

    expect(setQueryResults).toHaveBeenCalledWith("q1", "context", mockResults, {
      queries: [{ built: true }],
      stepTypes: ["select"],
      rowIds: ["r1"],
    });
  });

  it("does not store results when route params are missing", async () => {
    routeState.params = { owner: "alice" };

    await setup().runOrStopQuery();

    expect(setQueryResults).not.toHaveBeenCalled();
  });

  it("stores results after admin exec execution", async () => {
    routeState.name = "admin-db-view";

    await setup({ tab: "exec" }).runOrStopQuery();

    expect(setQueryResults).toHaveBeenCalledWith("q1", "exec", mockResults, {
      queries: [{ built: true }],
      stepTypes: ["select"],
      rowIds: ["r1"],
    });
  });

  describe("mutations opened from a link", () => {
    type ModalProps = {
      header: string;
      content: { paragraph: { text: string }[] }[];
      onConfirm: () => boolean;
    };
    const modalProps = (): ModalProps =>
      openModal.mock.calls[0]?.[0] as ModalProps;

    beforeEach(() => {
      queryState.unconfirmedLinkedMutation = true;
    });

    it("asks before running and shows the queries", async () => {
      const steps = [makeStep("select"), makeStep("select.aliases")];
      await setup({ tab: "exec_mut", steps }).runOrStopQuery();

      expect(runQuery).not.toHaveBeenCalled();
      expect(openModal).toHaveBeenCalledOnce();
      const texts = modalProps().content.map((c) =>
        c.paragraph.map((p) => p.text).join(""),
      );
      expect(texts[0]).toContain("change the database");
      expect(texts[1]).toBe("QueryBuilder::select().aliases().query()");
    });

    it("runs once confirmed and does not ask again", async () => {
      await setup({ tab: "exec_mut" }).runOrStopQuery();
      expect(modalProps().onConfirm()).toBe(true);
      await vi.waitFor(() => expect(db_exec_mut).toHaveBeenCalledOnce());
      expect(setUnconfirmedLinkedMutation).toHaveBeenCalledWith("q1", false);
    });

    it("runs what was shown even if the rows change while asking", async () => {
      const queryRows = ref<QueryRow[]>([
        { id: "r1", steps: [makeStep("select"), makeStep("select.aliases")] },
      ]);
      const execution = useQueryExecution(
        ref("q1"),
        ref<TAB>("exec_mut"),
        computed(() => queryRows.value),
        computed(() => false),
      );
      await execution.runOrStopQuery();
      queryRows.value = [{ id: "r2", steps: [makeStep("select")] }];

      modalProps().onConfirm();
      await vi.waitFor(() => expect(setQueryResults).toHaveBeenCalledOnce());
      // Built from the rows that were shown, not from the changed ones.
      expect(buildQueryFromSteps).toHaveBeenCalledOnce();
      expect(buildQueryFromSteps).toHaveBeenCalledWith([
        expect.objectContaining({ type: "select" }),
        expect.objectContaining({ type: "select.aliases" }),
      ]);
      expect(setQueryResults).toHaveBeenCalledWith(
        "q1",
        "exec_mut",
        mockResults,
        {
          queries: [{ built: true }],
          stepTypes: ["select", "select.aliases"],
          rowIds: ["r1"],
        },
      );
    });

    it("does not ask for other tabs or confirmed rows", async () => {
      await setup({ tab: "exec" }).runOrStopQuery();
      queryState.unconfirmedLinkedMutation = false;
      await setup({ tab: "exec_mut" }).runOrStopQuery();
      expect(openModal).not.toHaveBeenCalled();
      expect(db_exec).toHaveBeenCalledOnce();
      expect(db_exec_mut).toHaveBeenCalledOnce();
    });
  });
});
