import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, h, inject, nextTick, ref, type Ref } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});

const { executeDbQueries, visibility } = vi.hoisted(() => ({
  executeDbQueries: vi.fn(),
  visibility: { value: "visible" },
}));

vi.mock(
  "@agdb-studio/query/src/composables/dbExec",
  async (importOriginal) => ({
    ...(await importOriginal<
      typeof import("@agdb-studio/query/src/composables/dbExec")
    >()),
    executeDbQueries,
  }),
);
vi.mock("@vueuse/core", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@vueuse/core")>()),
  useDocumentVisibility: () => visibility,
}));

import { useMinimalRoute } from "@agdb-studio/query/src/composables/dbExec";
import { useAdmin } from "@agdb-studio/router/src/admin";
import { useQueryStore } from "@agdb-studio/query/src/composables/queryStore";
import { errorMessage, useWidgetQuery } from "./widgetQuery";
import { createWidget } from "../utils/dashboardDocument";
import type { Widget } from "../types";

const USERS = 'select().ids("users")';
const execResults = [{ result: 1, elements: [] }];
const contextResults = [{ result: 2, elements: [] }];

const setup = (init: Partial<Widget> = {}) => {
  const widget = ref<Widget>(
    createWidget({
      owner: "o",
      db: "d",
      rows: {
        exec: [{ id: "r1", query: USERS }],
        context: [{ id: "r2", query: "select().ids(1)" }],
      },
      ...init,
    }),
  );
  let data!: ReturnType<typeof useWidgetQuery>;
  let injected: {
    id?: Ref<string>;
    params?: Record<string, unknown>;
    name?: unknown;
  } = {};
  // Plain component objects: one defineComponent per file keeps lint happy.
  const Child = {
    setup() {
      const route = useMinimalRoute();
      injected = {
        id: inject<Ref<string>>("queryId"),
        params: route.params,
        name: route.name,
      };
      return () => null;
    },
  };
  const wrapper = mount(
    defineComponent({
      setup() {
        data = useWidgetQuery(widget);
        return () => h(Child);
      },
    }),
  );
  return { widget, data, wrapper, injected: () => injected };
};

describe("useWidgetQuery", () => {
  const queryStore = useQueryStore();

  beforeEach(() => {
    executeDbQueries.mockReset();
    executeDbQueries.mockImplementation(async (queries: unknown[]) =>
      queries.length === 1 && JSON.stringify(queries).includes("users")
        ? execResults
        : contextResults,
    );
    visibility.value = "visible";
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("runs the exec and context rows on the widget's database", async () => {
    const { data, injected, wrapper } = setup();
    expect(data.loading.value).toBe(true);
    await flushPromises();
    expect(executeDbQueries).toHaveBeenCalledTimes(2);
    expect(executeDbQueries).toHaveBeenCalledWith(
      [expect.anything()],
      false,
      { params: { owner: "o", db: "d" }, name: "query" },
      { quiet: false },
    );
    const query = data.query.value!;
    expect(query.results.exec).toEqual(execResults);
    expect(query.results.context).toEqual(contextResults);
    expect(query.executedRowIds).toEqual({ exec: ["r1"], context: ["r2"] });
    expect(query.lastExecutedTab).toBe("exec");
    expect(query.lastRun).toBeDefined();
    expect(data.loading.value).toBe(false);
    expect(injected().id?.value).toBe(data.queryId.value);
    expect(injected().params).toEqual({ owner: "o", db: "d" });
    expect(injected().name).toBe("query");
    wrapper.unmount();
    expect(queryStore.getQuery(data.queryId.value)).toBeUndefined();
  });

  it("runs with admin access in the admin view", async () => {
    const { isAdmin, isAdminView } = useAdmin();
    (isAdmin as Ref<boolean>).value = true;
    (isAdminView as Ref<boolean>).value = true;
    const { injected, wrapper } = setup();
    await flushPromises();
    expect(executeDbQueries).toHaveBeenCalledWith(
      [expect.anything()],
      false,
      { params: { owner: "o", db: "d" }, name: "admin-query" },
      { quiet: false },
    );
    expect(injected().name).toBe("admin-query");
    wrapper.unmount();
  });

  it("skips an empty context tab", async () => {
    const { data } = setup({
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
    });
    await flushPromises();
    expect(executeDbQueries).toHaveBeenCalledTimes(1);
    expect(data.query.value!.results.context).toEqual([]);
  });

  it("explains a query that cannot be built and stops auto refresh", async () => {
    vi.useFakeTimers();
    const { data } = setup({
      rows: { exec: [{ id: "r1", query: "select()" }], context: [] },
      refreshSeconds: 10,
    });
    await flushPromises();
    expect(data.error.value).toContain("terminal step");
    expect(data.loading.value).toBe(false);
    await vi.advanceTimersByTimeAsync(30_000);
    expect(executeDbQueries).not.toHaveBeenCalled();
  });

  it("asks for a query when the widget has none", async () => {
    const { data } = setup({ rows: { exec: [], context: [] } });
    await flushPromises();
    expect(executeDbQueries).not.toHaveBeenCalled();
    expect(data.error.value).toContain("No query yet");
    expect(data.loading.value).toBe(false);
  });

  it("shows errors and keeps only the newest run", async () => {
    let rejectFirst: (value: unknown) => void = () => {};
    executeDbQueries.mockImplementationOnce(
      () => new Promise((_, reject) => (rejectFirst = reject)),
    );
    const { widget, data } = setup({
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
    });
    widget.value = { ...widget.value, db: "other" };
    await flushPromises();
    rejectFirst({ response: { status: 403 } });
    await flushPromises();
    expect(data.error.value).toBeUndefined();

    executeDbQueries.mockRejectedValueOnce({ response: { status: 403 } });
    await data.run();
    expect(data.error.value).toBe("No access to this database");
  });

  it("ignores results of an older run that finishes last", async () => {
    let resolveFirst: (value: unknown) => void = () => {};
    executeDbQueries.mockImplementationOnce(
      () => new Promise((resolve) => (resolveFirst = resolve)),
    );
    const { data } = setup({
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
    });
    await flushPromises();
    await data.run();
    const lastRun = data.query.value!.lastRun;
    resolveFirst([{ result: 99, elements: [] }]);
    await flushPromises();
    expect(data.query.value!.results.exec).toEqual(execResults);
    expect(data.query.value!.lastRun).toBe(lastRun);
  });

  it.each([
    [{ response: { status: 403 } }, "No access to this database"],
    [{ response: { status: 404 } }, "Database not found"],
    [{ response: { status: 400, data: "bad query" } }, "bad query"],
    [new Error("network"), "network"],
    ["plain", "plain"],
  ])("explains errors (%#)", (error, message) => {
    expect(errorMessage(error)).toBe(message);
  });

  it("re-runs when the queries change, not the title, view or colours", async () => {
    const { widget, data } = setup();
    await flushPromises();
    const revision = data.revision.value;
    widget.value = { ...widget.value, title: "Renamed" };
    await flushPromises();
    expect(executeDbQueries).toHaveBeenCalledTimes(2);
    expect(data.revision.value).toBe(revision);

    widget.value = { ...widget.value, view: "table" };
    await flushPromises();
    expect(data.revision.value).toBe(revision + 1);
    expect(data.query.value!.view).toBe("table");
    widget.value = {
      ...widget.value,
      colors: { rules: [], rows: { r1: 2 }, elements: {} },
    };
    await flushPromises();
    expect(data.revision.value).toBe(revision + 2);
    expect(data.query.value!.colors?.rows).toEqual({ r1: 2 });
    // A chart edited elsewhere (the editor, another user) is shown too.
    const chart = {
      kind: "donut",
      groupBy: { by: "query" },
      measure: { aggregate: "count" },
    } as const;
    widget.value = { ...widget.value, chart };
    await flushPromises();
    expect(data.revision.value).toBe(revision + 3);
    expect(data.query.value!.chart).toEqual(chart);
    expect(data.displayChanged.value).toBe(false);
    expect(executeDbQueries).toHaveBeenCalledTimes(2);
  });

  it("tells when the view changed in the results and resets it", async () => {
    const { widget, data } = setup({ view: "graph" });
    await flushPromises();
    expect(data.displayChanged.value).toBe(false);
    expect(data.shownWidget()).toEqual(widget.value);
    queryStore.setView(data.queryId.value, "table");
    expect(data.displayChanged.value).toBe(true);
    expect(data.shownWidget().view).toBe("table");
    const revision = data.revision.value;
    data.resetDisplay();
    expect(data.revision.value).toBe(revision + 1);
    expect(data.query.value!.view).toBe("graph");
    expect(data.displayChanged.value).toBe(false);
  });

  it("notices nested colour changes in the results", async () => {
    const { data } = setup({
      colors: { rules: [], rows: { r1: 1 }, elements: {} },
    });
    await flushPromises();
    expect(data.displayChanged.value).toBe(false);
    queryStore.setElementColor(data.queryId.value, 7, 0);
    expect(data.displayChanged.value).toBe(true);
    queryStore.setElementColor(data.queryId.value, 7, undefined);
    expect(data.displayChanged.value).toBe(false);
    queryStore.setRowColor(data.queryId.value, "r1", 3);
    expect(data.displayChanged.value).toBe(true);
  });

  it("does not start over when the widget saves what its results show", async () => {
    const { widget, data } = setup();
    await flushPromises();
    queryStore.setView(data.queryId.value, "table");
    const revision = data.revision.value;
    widget.value = data.shownWidget();
    await flushPromises();
    expect(data.revision.value).toBe(revision);
    expect(data.displayChanged.value).toBe(false);
  });

  it("shows no results when the database returns none", async () => {
    executeDbQueries.mockResolvedValue(undefined);
    const { data } = setup({
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
    });
    await flushPromises();
    expect(data.query.value!.results.exec).toEqual([]);
  });

  it("stops auto refresh after a failure until a run succeeds", async () => {
    vi.useFakeTimers();
    executeDbQueries.mockRejectedValueOnce({ response: { status: 404 } });
    const { data } = setup({
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      refreshSeconds: 10,
    });
    await flushPromises();
    expect(data.error.value).toBe("Database not found");
    await vi.advanceTimersByTimeAsync(60_000);
    expect(executeDbQueries).toHaveBeenCalledTimes(1);
    // A manual refresh that works turns it back on.
    await data.run();
    expect(data.error.value).toBeUndefined();
    await vi.advanceTimersByTimeAsync(10_000);
    expect(executeDbQueries).toHaveBeenCalledTimes(3);
  });

  it.each([-5, 5, Number.POSITIVE_INFINITY, 86_401])(
    "does not refresh on a bad interval (%s seconds)",
    async (refreshSeconds) => {
      vi.useFakeTimers();
      setup({
        rows: { exec: [{ id: "r1", query: USERS }], context: [] },
        refreshSeconds,
      });
      await flushPromises();
      await vi.advanceTimersByTimeAsync(60_000);
      expect(executeDbQueries).toHaveBeenCalledTimes(1);
    },
  );

  it("refreshes on an interval while the tab is visible", async () => {
    vi.useFakeTimers();
    const { widget } = setup({
      rows: { exec: [{ id: "r1", query: USERS }], context: [] },
      refreshSeconds: 10,
    });
    await flushPromises();
    expect(executeDbQueries).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(10_000);
    expect(executeDbQueries).toHaveBeenCalledTimes(2);
    visibility.value = "hidden";
    await vi.advanceTimersByTimeAsync(10_000);
    expect(executeDbQueries).toHaveBeenCalledTimes(2);
    visibility.value = "visible";
    widget.value = { ...widget.value, refreshSeconds: undefined };
    await nextTick();
    await vi.advanceTimersByTimeAsync(30_000);
    expect(executeDbQueries).toHaveBeenCalledTimes(2);
  });
});
