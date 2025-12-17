import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { useQueryStore } from "./queryStore";
import type { Query, QueryStep } from "./types";

describe("queryStore", () => {
  const store = useQueryStore();

  const makeQuery = (overrides?: Partial<Query>): Query => ({
    id: overrides?.id ?? "q1",
    name: overrides?.name ?? "Query 1",
    steps: overrides?.steps ?? [],
    isRunning: overrides?.isRunning ?? false,
    lastRun: overrides?.lastRun,
  });

  const makeStep = (overrides?: Partial<QueryStep>): QueryStep => ({
    id: overrides?.id ?? "s1",
    type: overrides?.type ?? "select",
    name: overrides?.name ?? "Step 1",
    data: overrides?.data,
  });

  beforeEach(() => {
    store.clearQueries();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("adds and retrieves a query", () => {
    const q = makeQuery();
    store.addQuery(q);
    const got = store.getQuery(q.id);
    expect(got).toBeDefined();
    expect(got?.id).toBe(q.id);
    expect(got?.name).toBe("Query 1");
    expect(got?.steps).toEqual([]);
  });

  it("updates an existing query", () => {
    const q = makeQuery();
    store.addQuery(q);

    const updated = { ...q, name: "Updated Query" };
    store.updateQuery(updated);

    const got = store.getQuery(q.id);
    expect(got?.name).toBe("Updated Query");
  });

  it("deletes a query", () => {
    const q = makeQuery();
    store.addQuery(q);

    store.deleteQuery(q.id);
    expect(store.getQuery(q.id)).toBeUndefined();
  });

  it("adds a query step", () => {
    const q = makeQuery();
    store.addQuery(q);

    const step = makeStep();
    store.addQueryStep(q.id, step);

    const got = store.getQuery(q.id);
    expect(got?.steps.length).toBe(1);
    expect(got?.steps[0]).toEqual(step);
  });

  it("updates a query step", () => {
    const q = makeQuery();
    store.addQuery(q);

    const step = makeStep();
    store.addQueryStep(q.id, step);

    const updatedStep = { ...step, name: "Updated Step" };
    store.updateQueryStep(q.id, updatedStep);

    const got = store.getQuery(q.id);
    expect(got?.steps[0]?.name).toBe("Updated Step");
  });

  it("deletes a query step", () => {
    const q = makeQuery();
    store.addQuery(q);

    const s1 = makeStep({ id: "s1" });
    const s2 = makeStep({ id: "s2", name: "Step 2" });
    store.addQueryStep(q.id, s1);
    store.addQueryStep(q.id, s2);

    store.deleteQueryStep(q.id, "s1");
    const got = store.getQuery(q.id);
    expect(got?.steps.length).toBe(1);
    expect(got?.steps[0]?.id).toBe("s2");
  });

  it("runs a query and sets lastRun after completion", () => {
    vi.useFakeTimers();

    const q = makeQuery();
    store.addQuery(q);

    store.runQuery(q.id);
    // Immediately marked as running
    expect(store.getQuery(q.id)?.isRunning).toBe(true);

    // Advance simulated time to complete execution
    vi.advanceTimersByTime(1000);

    const got = store.getQuery(q.id);
    expect(got?.isRunning).toBe(false);
    expect(got?.lastRun).toBeInstanceOf(Date);
  });

  it("stops a running query", () => {
    vi.useFakeTimers();

    const q = makeQuery();
    store.addQuery(q);

    store.runQuery(q.id);
    expect(store.getQuery(q.id)?.isRunning).toBe(true);

    store.stopQuery(q.id);
    expect(store.getQuery(q.id)?.isRunning).toBe(false);
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

    // updateQuery with missing
    store.updateQuery(makeQuery({ id: missingId }));
    expect(store.getQuery(missingId)).toBeUndefined();

    // deleteQuery with missing
    store.deleteQuery(missingId);
    expect(store.getQuery(missingId)).toBeUndefined();

    // step operations with missing
    store.addQueryStep(missingId, step);
    store.updateQueryStep(missingId, step);
    store.deleteQueryStep(missingId, step.id);
    expect(store.getQuery(missingId)).toBeUndefined();

    // run/stop missing
    vi.useFakeTimers();
    store.runQuery(missingId);
    vi.advanceTimersByTime(1000);
    store.stopQuery(missingId);
    expect(store.getQuery(missingId)).toBeUndefined();
  });

  it("does not update step when id not found", () => {
    const q = makeQuery({ id: "qid" });
    store.addQuery(q);

    const existing = makeStep({ id: "s1" });
    store.addQueryStep(q.id, existing);

    const nonExisting = makeStep({ id: "s2", name: "Should Not Apply" });
    store.updateQueryStep(q.id, nonExisting);

    const got = store.getQuery(q.id);
    expect(got?.steps.length).toBe(1);
    expect(got?.steps[0]?.id).toBe("s1");
    expect(got?.steps[0]?.name).not.toBe("Should Not Apply");
  });
});
