import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { useQueryStore } from "./queryStore";
import type { AddQueryParams, Query, QueryStep } from "./types";

describe("queryStore", () => {
  const store = useQueryStore();

  const makeQuery = (overrides?: Partial<Query>): AddQueryParams => ({
    id: overrides?.id ?? "q1",
    name: overrides?.name ?? "Query 1",
  });

  const makeStep = (overrides?: Partial<QueryStep>): QueryStep => ({
    id: overrides?.id ?? "s1",
    type: overrides?.type ?? "select",
    values: overrides?.values ?? [],
  });

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
    expect(got?.value?.steps.exec).toEqual([]);
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
    store.addQueryStep(queryRef.value.id, "exec", step);

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.steps.exec.length).toBe(1);
    expect(got?.value?.steps.exec[0]).toEqual(step);
  });

  it("updates a query step", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const step = makeStep();
    store.addQueryStep(queryRef.value.id, "exec", step);

    const updatedStep = { ...step, name: "Updated Step" };
    store.updateQueryStep(queryRef.value.id, "exec", updatedStep);
    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.steps.exec.length).toBe(1);
    expect(got?.value?.steps.exec[0]).toEqual(updatedStep);
  });

  it("deletes a query step", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const s1 = makeStep({ id: "s1" });
    const s2 = makeStep({ id: "s2" });
    store.addQueryStep(queryRef.value.id, "exec", s1);
    store.addQueryStep(queryRef.value.id, "exec", s2);

    store.deleteQueryStep(queryRef.value.id, "exec", "s1");
    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.steps.exec.length).toBe(1);
    expect(got?.value?.steps.exec[0]?.id).toBe("s2");
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
    store.addQueryStep(missingId, "exec", step);
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
    store.addQueryStep(queryRef.value.id, "exec", existing);

    const nonExisting = makeStep({ id: "s2" });
    store.updateQueryStep(queryRef.value.id, "exec", nonExisting);

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.steps.exec.length).toBe(1);
    expect(got?.value?.steps.exec[0]?.id).toBe("s1");
  });

  it("clears query steps for a given tab", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const s1 = makeStep({ id: "s1" });
    const s2 = makeStep({ id: "s2" });
    store.addQueryStep(queryRef.value.id, "exec", s1);
    store.addQueryStep(queryRef.value.id, "exec", s2);

    store.clearQuerySteps(queryRef.value.id, "exec");
    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.steps.exec.length).toBe(0);
  });

  it("should handle clearing steps for missing query gracefully", () => {
    const missingId = "missing";
    store.clearQuerySteps(missingId, "exec");
    expect(store.getQuery(missingId)).toBeUndefined();
  });

  it("validates query steps based on followers", () => {
    const q = makeQuery();
    const queryRef = store.addQuery(q);

    const step1 = makeStep({ id: "s1", type: "select" });
    const step2 = makeStep({ id: "s2", type: "search" });
    const step3 = makeStep({ id: "s3", type: "insert" }); // invalid after search

    store.addQueryStep(queryRef.value.id, "exec", step1);
    store.addQueryStep(queryRef.value.id, "exec", step2);
    store.addQueryStep(queryRef.value.id, "exec", step3);
    store.validateQuerySteps(queryRef.value.id, "exec");

    const got = store.getQuery(queryRef.value.id);
    expect(got?.value?.steps.exec.length).toBe(3);
    expect(got?.value?.steps.exec[0]?.invalid).toBe(false); // select valid
    expect(got?.value?.steps.exec[1]?.invalid).toBe(false); // search valid
    expect(got?.value?.steps.exec[2]?.invalid).toBe(true); // insert invalid
  });

  it("should handle validation for missing query gracefully", () => {
    const missingId = "missing";
    store.validateQuerySteps(missingId, "exec");
    expect(store.getQuery(missingId)).toBeUndefined();
  });
});
