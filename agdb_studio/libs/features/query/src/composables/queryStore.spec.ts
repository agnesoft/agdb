import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { useQueryStore } from "./queryStore";
import type { AddQueryParams, Query, QueryStep, TAB } from "./types";
import { queryApiMock } from "../mock/queryApiMock";

describe("queryStore", () => {
  const store = useQueryStore();

  const makeQuery = (overrides?: Partial<Query>): AddQueryParams => ({
    id: overrides?.id ?? "q1",
    name: overrides?.name ?? "Query 1",
  });

  const makeStep = (overrides?: Partial<QueryStep>): QueryStep => ({
    id: overrides?.id ?? "s1",
    type: overrides?.type ?? "select",
  });

  const firstRowId = (queryId: string, tab: TAB = "exec"): string =>
    store.getQuery(queryId)?.value.rows[tab][0]?.id ?? "missing-row";

  beforeEach(() => {
    store.clearQueries();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("adds and retrieves a query", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);
    const got = store.getQuery(queryRef.value.id);
    expect(got).toBeDefined();
    expect(got?.value?.id).toBe(queryRef.value.id);
    expect(got?.value?.name).toBe("Query 1");
    expect(got?.value?.rows.exec).toHaveLength(1);
    expect(got?.value?.rows.exec[0]?.steps).toEqual([]);
  });

  it("deletes a query", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    store.deleteQuery(queryRef.value.id);
    expect(store.getQuery(queryRef.value.id)).toBeUndefined();
  });

  it("adds a query step", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const step = makeStep();
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      step,
    );

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec[0]?.steps.length).toBe(1);
    expect(got?.value?.rows.exec[0]?.steps[0]).toEqual(step);
  });

  it("updates a query step", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const step = makeStep();
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      step,
    );

    const updatedStep = { ...step, name: "Updated Step" };
    store.updateQueryStep(queryRef.value.id, "exec", updatedStep);
    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec[0]?.steps.length).toBe(1);
    expect(got?.value?.rows.exec[0]?.steps[0]).toEqual(updatedStep);
  });

  it("deletes a query step", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const s1 = makeStep({ id: "s1" });
    const s2 = makeStep({ id: "s2" });
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      s1,
    );
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      s2,
    );

    store.deleteQueryStep(queryRef.value.id, "exec", "s1");
    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec[0]?.steps.length).toBe(1);
    expect(got?.value?.rows.exec[0]?.steps[0]?.id).toBe("s2");
  });

  it("runs a query and sets lastRun after completion", () => {
    vi.useFakeTimers();

    const q = makeQuery();
    const queryRef = store.addQuery(q);

    store.runQuery(queryRef.value.id);
    // Immediately marked as running
    expect(store.getQuery(queryRef.value.id)?.value?.isRunning).toBe(true);
    // Advance simulated time to complete execution
    vi.advanceTimersByTime(1000);

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.isRunning).toBe(false);
    expect(got?.value?.lastRun).toBeInstanceOf(Date);
  });

  it("runs a query with an async runner and records completion", async () => {
    const q = makeQuery({ id: "runner-query" });
    const queryRef = store.addQuery(q);
    const runner = vi.fn().mockResolvedValue(undefined);

    await store.runQuery(queryRef.value.id, runner);

    const got = store.getQuery(queryRef.value.id);
    expect(runner).toHaveBeenCalledOnce();
    expect(got?.value?.isRunning).toBe(false);
    expect(got?.value?.lastRun).toBeInstanceOf(Date);
  });

  it("records completion even when an async runner fails", async () => {
    const q = makeQuery({ id: "runner-error-query" });
    const queryRef = store.addQuery(q);
    const runner = vi.fn().mockRejectedValue(new Error("runner failed"));

    await expect(store.runQuery(queryRef.value.id, runner)).rejects.toThrow(
      "runner failed",
    );

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.isRunning).toBe(false);
    expect(got?.value?.lastRun).toBeInstanceOf(Date);
  });

  it("stops a running query", () => {
    vi.useFakeTimers();

    const q = makeQuery();
    const queryRef = store.addQuery(q);

    store.runQuery(queryRef.value.id);
    expect(store.getQuery(queryRef.value.id)?.value?.isRunning).toBe(true);
    store.stopQuery(queryRef.value.id);
    expect(store.getQuery(queryRef.value.id)?.value?.isRunning).toBe(false);
  });

  it("clears all queries", () => {
    const q1 = makeQuery({ id: "q1" });
    const q2 = makeQuery({ id: "q2", name: "Query 2" });
    store.addQuery(q1);
    store.addQuery(q2);

    store.clearQueries();
    expect(store.getQuery("q1")).toBeUndefined();
    expect(store.getQuery("q2")).toBeUndefined();
  });

  it("gracefully handles operations on missing queries", () => {
    // none added; operations should be no-ops
    const missingId = "missing";
    const step = makeStep({ id: "x" });

    // deleteQuery with missing
    store.deleteQuery(missingId);
    expect(store.getQuery(missingId)).toBeUndefined();

    // step operations with missing
    store.addQueryStep(missingId, "exec", firstRowId(missingId, "exec"), step);
    store.updateQueryStep(missingId, "exec", step);
    store.deleteQueryStep(missingId, "exec", step.id);
    expect(store.getQuery(missingId)).toBeUndefined();

    // run/stop missing
    vi.useFakeTimers();
    store.runQuery(missingId);
    vi.advanceTimersByTime(1000);
    store.stopQuery(missingId);
    expect(store.getQuery(missingId)).toBeUndefined();
  });

  it("does not update step when id not found", () => {
    const qid = "qid";
    const q = makeQuery({ id: qid });
    const queryRef = store.addQuery(q);

    const existing = makeStep({ id: "s1" });
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      existing,
    );

    const nonExisting = makeStep({ id: "s2" });
    store.updateQueryStep(queryRef.value.id, "exec", nonExisting);

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec[0]?.steps.length).toBe(1);
    expect(got?.value?.rows.exec[0]?.steps[0]?.id).toBe("s1");
  });

  it("clears query steps for a given tab", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const s1 = makeStep({ id: "s1" });
    const s2 = makeStep({ id: "s2" });
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      s1,
    );
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      s2,
    );

    store.clearQuerySteps(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
    );
    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec[0]?.steps.length).toBe(0);
  });

  it("should handle clearing steps for missing query gracefully", () => {
    const missingId = "missing";
    store.clearQuerySteps(missingId, "exec", firstRowId(missingId, "exec"));
    expect(store.getQuery(missingId)).toBeUndefined();
  });

  it("validates query steps based on followers", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const step1 = makeStep({ id: "s1", type: "select" });
    const step2 = makeStep({ id: "s2", type: "search" });
    const step3 = makeStep({ id: "s3", type: "insert" }); // invalid after search

    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      step1,
    );
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      step2,
    );
    store.addQueryStep(
      queryRef.value.id,
      "exec",
      firstRowId(queryRef.value.id, "exec"),
      step3,
    );
    store.validateQuerySteps(queryRef.value.id, "exec");

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec[0]?.steps.length).toBe(3);
    expect(got?.value?.rows.exec[0]?.steps[0]?.invalid).toBe(false); // select valid
    expect(got?.value?.rows.exec[0]?.steps[1]?.invalid).toBe(false); // search valid
    expect(got?.value?.rows.exec[0]?.steps[2]?.invalid).toBe(true); // insert invalid
  });

  it("should handle validation for missing query gracefully", () => {
    const missingId = "missing";
    store.validateQuerySteps(missingId, "exec");
    expect(store.getQuery(missingId)).toBeUndefined();
  });

  it("validates query steps for exec_mut tab", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const step1 = makeStep({ id: "s1", type: "insert" });
    const step2 = makeStep({ id: "s2", type: "insert.values" });
    const step3 = makeStep({ id: "s3", type: "select" }); // invalid after values in exec_mut

    store.addQueryStep(
      queryRef.value.id,
      "exec_mut",
      firstRowId(queryRef.value.id, "exec_mut"),
      step1,
    );
    store.addQueryStep(
      queryRef.value.id,
      "exec_mut",
      firstRowId(queryRef.value.id, "exec_mut"),
      step2,
    );
    store.addQueryStep(
      queryRef.value.id,
      "exec_mut",
      firstRowId(queryRef.value.id, "exec_mut"),
      step3,
    );
    store.validateQuerySteps(queryRef.value.id, "exec_mut");

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.rows.exec_mut[0]?.steps[0]?.invalid).toBe(false); // insert valid
    expect(got?.value?.rows.exec_mut[0]?.steps[1]?.invalid).toBe(false); // values valid
    expect(got?.value?.rows.exec_mut[0]?.steps[2]?.invalid).toBe(true); // select invalid
  });

  it("initializes a new query with empty results", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);
    expect(queryRef.value.results).toEqual({});
    expect(queryRef.value.lastExecutedTab).toBeUndefined();
  });

  it("stores query results for exec tab and sets lastExecutedTab", () => {
    const q = makeQuery();
    store.addQuery(q);

    const results = [{ result: 2, elements: [] }];
    store.setQueryResults("q1", "exec", results);

    const got = store.getQuery("q1");
    expect(got?.value?.results.exec).toEqual(results);
    expect(got?.value?.lastExecutedTab).toBe("exec");
  });

  it("stores query results for exec_mut tab and sets lastExecutedTab", () => {
    const q = makeQuery();
    store.addQuery(q);

    const results = [{ result: 1, elements: [] }];
    store.setQueryResults("q1", "exec_mut", results);

    const got = store.getQuery("q1");
    expect(got?.value?.results.exec_mut).toEqual(results);
    expect(got?.value?.lastExecutedTab).toBe("exec_mut");
  });

  it("stores query results for context tab without changing lastExecutedTab", () => {
    const q = makeQuery();
    store.addQuery(q);

    const results = [{ result: 5, elements: [] }];
    store.setQueryResults("q1", "context", results);

    const got = store.getQuery("q1");
    expect(got?.value?.results.context).toEqual(results);
    expect(got?.value?.lastExecutedTab).toBeUndefined();
  });

  it("is a no-op when setting results for missing query", () => {
    store.setQueryResults("missing", "exec", [{ result: 0, elements: [] }]);
    expect(store.getQuery("missing")).toBeUndefined();
  });

  it("clears results for a specific tab", () => {
    const q = makeQuery();
    store.addQuery(q);

    store.setQueryResults("q1", "exec", [{ result: 1, elements: [] }]);
    store.setQueryResults("q1", "context", [{ result: 2, elements: [] }]);

    store.clearQueryResults("q1", "exec");

    const got = store.getQuery("q1");
    expect(got?.value?.results.exec).toBeUndefined();
    expect(got?.value?.results.context).toBeDefined();
  });

  it("clears all results and resets lastExecutedTab", () => {
    const q = makeQuery();
    store.addQuery(q);

    store.setQueryResults("q1", "exec", [{ result: 1, elements: [] }]);
    store.setQueryResults("q1", "context", [{ result: 2, elements: [] }]);

    store.clearQueryResults("q1");

    const got = store.getQuery("q1");
    expect(got?.value?.results).toEqual({});
    expect(got?.value?.lastExecutedTab).toBeUndefined();
  });

  it("is a no-op when clearing results for missing query", () => {
    store.clearQueryResults("missing", "exec");
    store.clearQueryResults("missing");
    expect(store.getQuery("missing")).toBeUndefined();
  });

  it("falls back to empty followers when a followed step is missing from the mock map", () => {
    const q = makeQuery({ id: "qid-fallback" });
    const queryRef = store.addQuery(q);

    const originalFollowers = [...(queryApiMock.exec?.followers ?? [])];
    if (queryApiMock.exec) {
      queryApiMock.exec.followers = [
        "mystery.step",
      ] as typeof queryApiMock.exec.followers;
    }

    try {
      store.addQueryStep(
        queryRef.value.id,
        "exec",
        firstRowId(queryRef.value.id, "exec"),
        {
          id: "m1",
          type: "mystery.step" as QueryStep["type"],
        },
      );
      store.validateQuerySteps(queryRef.value.id, "exec");

      const got = store.getQuery(queryRef.value.id);
      expect(got?.value.rows.exec[0]?.steps[0]?.invalid).toBe(false);
    } finally {
      if (queryApiMock.exec) {
        queryApiMock.exec.followers = originalFollowers;
      }
    }
  });
  it("adds a new empty row to a tab", () => {
    store.addQuery(makeQuery());
    const row = store.addQueryRow("q1", "exec");

    const rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows).toHaveLength(2);
    expect(rows?.[1]).toEqual(row);
    expect(row?.steps).toEqual([]);
  });

  it("returns undefined when adding a row to a missing query", () => {
    expect(store.addQueryRow("missing", "exec")).toBeUndefined();
  });

  it("adds steps to a specific row only", () => {
    store.addQuery(makeQuery());
    const second = store.addQueryRow("q1", "exec");
    store.addQueryStep("q1", "exec", second?.id ?? "", makeStep());

    const rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows?.[0]?.steps).toHaveLength(0);
    expect(rows?.[1]?.steps).toHaveLength(1);
  });

  it("ignores adding or clearing steps on an unknown row", () => {
    store.addQuery(makeQuery());
    store.addQueryStep("q1", "exec", "nope", makeStep());
    store.clearQuerySteps("q1", "exec", "nope");
    expect(store.getQuery("q1")?.value.rows.exec[0]?.steps).toHaveLength(0);
  });

  it("updates and deletes steps in whichever row contains them", () => {
    store.addQuery(makeQuery());
    const second = store.addQueryRow("q1", "exec");
    const secondId = second?.id ?? "";
    store.addQueryStep("q1", "exec", secondId, makeStep({ id: "a" }));
    store.addQueryStep("q1", "exec", secondId, makeStep({ id: "b" }));

    store.updateQueryStep("q1", "exec", { id: "a", type: "search" });
    let rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows?.[1]?.steps[0]?.type).toBe("search");

    store.deleteQueryStep("q1", "exec", "a");
    rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows?.[1]?.steps.map((s) => s.id)).toEqual(["b"]);
  });

  it("ignores updating or deleting an unknown step id", () => {
    store.addQuery(makeQuery());
    store.addQueryStep("q1", "exec", firstRowId("q1"), makeStep({ id: "a" }));

    store.updateQueryStep("q1", "exec", makeStep({ id: "nope" }));
    store.deleteQueryStep("q1", "exec", "nope");
    expect(store.getQuery("q1")?.value.rows.exec[0]?.steps).toEqual([
      expect.objectContaining({ id: "a", type: "select" }),
    ]);
  });

  it("deletes a row", () => {
    store.addQuery(makeQuery());
    const second = store.addQueryRow("q1", "exec");
    const first = firstRowId("q1");

    store.deleteQueryRow("q1", "exec", first);
    const rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows).toHaveLength(1);
    expect(rows?.[0]?.id).toBe(second?.id);
  });

  it("keeps one empty row when the last row is deleted", () => {
    store.addQuery(makeQuery());
    const first = firstRowId("q1");
    store.addQueryStep("q1", "exec", first, makeStep());

    store.deleteQueryRow("q1", "exec", first);
    const rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows).toHaveLength(1);
    expect(rows?.[0]?.id).not.toBe(first);
    expect(rows?.[0]?.steps).toEqual([]);
  });

  it("is a no-op when deleting a row of a missing query", () => {
    store.deleteQueryRow("missing", "exec", "r1");
    expect(store.getQuery("missing")).toBeUndefined();
  });

  it("moves a row to a new position", () => {
    store.addQuery(makeQuery());
    const ids = [
      firstRowId("q1"),
      store.addQueryRow("q1", "exec")!.id,
      store.addQueryRow("q1", "exec")!.id,
    ];
    const rowIds = () => store.getQuery("q1")?.value.rows.exec.map((r) => r.id);

    store.moveQueryRow("q1", "exec", 0, 2);
    expect(rowIds()).toEqual([ids[1], ids[2], ids[0]]);

    store.moveQueryRow("q1", "exec", 2, 0);
    expect(rowIds()).toEqual(ids);
  });

  it("ignores moving to the same or an out-of-range index, or in a missing query", () => {
    store.addQuery(makeQuery());
    const ids = [firstRowId("q1"), store.addQueryRow("q1", "exec")!.id];
    const rowIds = () => store.getQuery("q1")?.value.rows.exec.map((r) => r.id);

    store.moveQueryRow("q1", "exec", 1, 1);
    store.moveQueryRow("q1", "exec", -1, 0);
    store.moveQueryRow("q1", "exec", 0, -1);
    store.moveQueryRow("q1", "exec", 2, 0);
    store.moveQueryRow("q1", "exec", 0, 2);
    store.moveQueryRow("missing", "exec", 0, 1);
    expect(rowIds()).toEqual(ids);
  });

  it("validates each row independently", () => {
    store.addQuery(makeQuery());
    const second = store.addQueryRow("q1", "exec");
    // Row 0: "insert" cannot follow "search", so its second step is invalid.
    store.addQueryStep(
      "q1",
      "exec",
      firstRowId("q1"),
      makeStep({ id: "s1", type: "search" }),
    );
    store.addQueryStep(
      "q1",
      "exec",
      firstRowId("q1"),
      makeStep({ id: "s2", type: "insert" }),
    );
    // Row 1 starts fresh, so "select" is valid at its start.
    store.addQueryStep(
      "q1",
      "exec",
      second?.id ?? "",
      makeStep({ id: "s3", type: "select" }),
    );

    const rows = store.getQuery("q1")?.value.rows.exec;
    expect(rows?.[0]?.steps[1]?.invalid).toBe(true);
    expect(rows?.[1]?.steps[0]?.invalid).toBe(false);
  });
});
