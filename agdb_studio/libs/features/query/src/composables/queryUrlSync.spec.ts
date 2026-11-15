import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick } from "vue";
import { URL_UPDATE_DELAY, useQueryUrlSync } from "./queryUrlSync";
import { useQueryStore } from "./queryStore";
import { MAX_QUERY_URL_LENGTH } from "../utils/queryUrl";

const { addNotification } = vi.hoisted(() => ({ addNotification: vi.fn() }));
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

const { useRoute, useRouter, onBeforeRouteLeave } = vi.hoisted(() => ({
  useRoute: vi.fn(),
  useRouter: vi.fn(),
  onBeforeRouteLeave: vi.fn(),
}));
vi.mock("vue-router", () => ({ useRoute, useRouter, onBeforeRouteLeave }));

describe("useQueryUrlSync", () => {
  const store = useQueryStore();
  const replace = vi.fn();
  let route: { query: Record<string, unknown> };

  const setup = () => {
    const scope = effectScope();
    const sync = scope.run(() => useQueryUrlSync());
    return { scope, sync: sync as ReturnType<typeof useQueryUrlSync> };
  };

  const rows = () => store.getQuery("q1")?.value.rows;

  beforeEach(() => {
    vi.useFakeTimers();
    store.clearQueries();
    store.addQuery({ id: "q1" });
    replace.mockReset();
    addNotification.mockReset();
    onBeforeRouteLeave.mockReset();
    route = { query: {} };
    useRoute.mockImplementation(() => route);
    useRouter.mockImplementation(() => ({ replace }));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("restores the rows from the URL", () => {
    route.query = {
      exec: ["select().aliases()", "select().ids(1)"],
      context: "select().ids(1)",
    };
    setup().sync.start("q1");
    expect(rows()?.exec.map((r) => r.steps.map((s) => s.type))).toEqual([
      ["select", "select.aliases"],
      ["select", "select.ids"],
    ]);
    expect(rows()?.context[0]?.steps).toHaveLength(2);
    expect(rows()?.exec_mut[0]?.steps).toEqual([]);
  });

  it("writes edited rows to the URL, keeping other params", async () => {
    route.query = { exec: "select().aliases()", other: "x" };
    setup().sync.start("q1");
    store.setQueryRows("q1", "context", [
      [{ type: "select" }, { type: "select.aliases" }],
    ]);
    store.setQueryRows("q1", "exec", []);
    await nextTick();
    expect(replace).not.toHaveBeenCalled();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).toHaveBeenCalledOnce();
    expect(replace).toHaveBeenCalledWith({
      query: { other: "x", context: ["select().aliases()"] },
    });
  });

  it("does not update the URL when it already matches", async () => {
    route.query = { exec: "select().aliases()" };
    setup().sync.start("q1");
    // Restoring sets the same rows again.
    store.setQueryRows("q1", "exec", [
      [{ type: "select" }, { type: "select.aliases" }],
    ]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).not.toHaveBeenCalled();
  });

  it("stops updating when stopped or disposed", async () => {
    const { scope, sync } = setup();
    sync.start("q1");
    store.setQueryRows("q1", "exec", [
      [{ type: "select" }, { type: "select.aliases" }],
    ]);
    await nextTick();
    scope.stop();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    store.setQueryRows("q1", "exec", []);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).not.toHaveBeenCalled();
  });

  it("does nothing for unknown queries", async () => {
    route.query = { exec: "select().aliases()" };
    setup().sync.start("nope");
    expect(rows()?.exec[0]?.steps).toEqual([]);
    store.setQueryRows("q1", "exec", [[{ type: "select" }]]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).not.toHaveBeenCalled();
  });

  it("does not update the URL once the query is gone", async () => {
    setup().sync.start("q1");
    store.setQueryRows("q1", "exec", [
      [{ type: "select" }, { type: "select.aliases" }],
    ]);
    await nextTick();
    store.deleteQuery("q1");
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).not.toHaveBeenCalled();
  });

  it("cancels a pending URL update when leaving the page", async () => {
    setup().sync.start("q1");
    store.setQueryRows("q1", "exec", [[{ type: "select" }]]);
    await nextTick();
    const leave = onBeforeRouteLeave.mock.calls[0]?.[0] as () => void;
    leave();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).not.toHaveBeenCalled();
  });

  it("warns about link values that cannot be read", () => {
    route.query = { exec: ["select().aliases()", "nope"], context: "nope" };
    setup().sync.start("q1");
    expect(rows()?.exec).toHaveLength(1);
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({
        type: "warning",
        message: "2 queries in the link could not be read.",
      }),
    );
  });

  it("opens no mutations when any of them cannot be read", () => {
    route.query = {
      exec: "select().aliases()",
      exec_mut: ["insert().nodes().count(1)", "nope"],
    };
    setup().sync.start("q1");
    expect(rows()?.exec[0]?.steps).toHaveLength(2);
    expect(rows()?.exec_mut[0]?.steps).toEqual([]);
    expect(store.getQuery("q1")?.value.unconfirmedLinkedMutation).toBeFalsy();
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({
        message:
          "1 query in the link could not be read. The exec_mut queries were not opened.",
      }),
    );
  });

  it("does nothing without a router", async () => {
    useRouter.mockReturnValue(undefined);
    useRoute.mockReturnValue(undefined);
    setup().sync.start("q1");
    store.setQueryRows("q1", "exec", [[{ type: "select" }]]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).not.toHaveBeenCalled();
    expect(onBeforeRouteLeave).not.toHaveBeenCalled();
  });

  const longValue = () =>
    `select().ids(${Array.from({ length: MAX_QUERY_URL_LENGTH / 2 }, (_, i) => i + 1).join(", ")})`;

  it("marks mutations from a link as unconfirmed", () => {
    route.query = { exec_mut: "insert().nodes().count(1)" };
    setup().sync.start("q1");
    expect(store.getQuery("q1")?.value.unconfirmedLinkedMutation).toBe(true);
  });

  it("does not mark links without mutations", () => {
    route.query = { exec: "select().aliases()" };
    setup().sync.start("q1");
    expect(store.getQuery("q1")?.value.unconfirmedLinkedMutation).toBeFalsy();
  });

  it("does not open links that are too long", () => {
    route.query = { exec: longValue() };
    setup().sync.start("q1");
    expect(rows()?.exec[0]?.steps).toEqual([]);
    expect(addNotification).toHaveBeenCalledWith(
      expect.objectContaining({ type: "warning", title: "Link too long" }),
    );
  });

  it("drops rows that are too long from the URL and warns once", async () => {
    route.query = { exec: "select().aliases()", other: "x" };
    setup().sync.start("q1");
    const steps = urlLong();
    store.setQueryRows("q1", "exec", [steps]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(replace).toHaveBeenCalledWith({ query: { other: "x" } });
    expect(addNotification).toHaveBeenCalledOnce();

    // Still too long: no new warning.
    route.query = { other: "x" };
    store.setQueryRows("q1", "exec", [steps, steps]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(addNotification).toHaveBeenCalledOnce();

    // Short again, then too long again: warns again.
    store.setQueryRows("q1", "exec", [[{ type: "select" }]]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    store.setQueryRows("q1", "exec", [steps]);
    await nextTick();
    vi.advanceTimersByTime(URL_UPDATE_DELAY);
    expect(addNotification).toHaveBeenCalledTimes(2);
  });

  /** `select().ids(1, 2, …)` with enough ids to go over the limit. */
  const urlLong = () => [
    { type: "select" as const },
    {
      type: "select.ids" as const,
      args: Array.from({ length: MAX_QUERY_URL_LENGTH / 2 }, (_, i) => [
        { selectedOption: "signed", value: `${i + 1}` },
      ]),
    },
  ];
});
