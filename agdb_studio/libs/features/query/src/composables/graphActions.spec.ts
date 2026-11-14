import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import {
  defineComponent,
  effectScope,
  h,
  nextTick,
  ref,
  type EffectScope,
} from "vue";
import { mount } from "@vue/test-utils";
import type { GraphElement, GraphLink, GraphNode } from "./types";

const { exec, execMut, openModal, addNotification, elementValues } = vi.hoisted(
  () => ({
    exec: vi.fn(),
    execMut: vi.fn(),
    openModal: vi.fn(),
    addNotification: vi.fn(),
    // The stored-values cache has its own spec; here only the wiring matters.
    elementValues: {
      load: vi.fn(async () => {}),
      set: vi.fn(),
      remove: vi.fn(),
      clear: vi.fn(),
      describe: vi.fn((element: GraphElement): GraphElement => element),
    },
  }),
);

vi.mock("./dbExec", () => ({
  useDbExec: () => ({ exec, execMut, route: { params: {} } }),
}));
vi.mock("./elementValues", () => ({
  PREFETCH_LIMIT: 2,
  useElementValues: () => elementValues,
}));
vi.mock("@agdb-studio/common/src/composables/modal/modal", () => ({
  default: () => ({ openModal }),
}));
vi.mock("@agdb-studio/notification/src/composables/notificationStore", () => ({
  addNotification,
}));

import {
  createGraphActions,
  elementLabel,
  isNode,
  provideGraphActions,
  useGraphActions,
} from "./graphActions";
import { useQueryStore } from "./queryStore";
import { QueryBuilder, type Components } from "@agnesoft/agdb_api";

const store = useQueryStore();

const dbNode = (id: number, values: unknown[] = []) => ({
  id,
  from: 0,
  to: 0,
  values: values as never[],
});
const dbEdge = (id: number, from: number, to: number) => ({
  id,
  from,
  to,
  values: [],
});

const graphNode = (id: number): GraphNode => ({
  id,
  values: [],
  rawValues: [],
  source: "query",
});
const graphLink = (id: number): GraphLink => ({
  id,
  values: [],
  rawValues: [],
  source: 1,
  target: 2,
  elementSource: "query",
});

/** The query that was actually sent by the last run. */
const EXECUTED_QUERY: Components.Schemas.QueryType = QueryBuilder.select()
  .ids([1, 2])
  .query();
const CONTEXT_QUERY: Components.Schemas.QueryType = QueryBuilder.select()
  .ids(7)
  .query();

const initialResults = () => [
  { result: 3, elements: [dbNode(1), dbNode(2), dbEdge(-3, 1, 2)] },
];

const setupQuery = (lastTab: "exec" | "exec_mut" = "exec") => {
  store.clearQueries();
  const query = store.addQuery({ id: "q1" });
  query.value.rows.exec = [{ id: "r", steps: [{ id: "s", type: "select" }] }];
  store.setQueryResults("q1", lastTab, initialResults(), {
    queries: [EXECUTED_QUERY],
  });
  query.value.lastRun = new Date(1);
  return query;
};

/** A promise that the test resolves or rejects by hand. */
const deferred = <T>() => {
  let resolve: (value: T) => void = () => {};
  let reject: (error: unknown) => void = () => {};
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
};

/**
 * Creates actions in an effect scope that is stopped after each test, as a
 * component unmount would; otherwise their watchers keep reacting to the
 * shared store in later tests.
 */
const scopes: EffectScope[] = [];
const createActions = (...args: Parameters<typeof createGraphActions>) => {
  const scope = effectScope();
  scopes.push(scope);
  const actions = scope.run(() => createGraphActions(...args));
  if (!actions) throw new Error("scope did not run");
  return actions;
};
afterEach(() => {
  scopes.splice(0).forEach((scope) => scope.stop());
});

const confirmModal = async () => {
  const { onConfirm } = openModal.mock.calls.at(-1)?.[0] ?? {};
  return await onConfirm();
};

describe("graphActions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // By default re-running the query returns the same elements.
    exec.mockImplementation(async () => initialResults());
    execMut.mockResolvedValue([{ result: 1, elements: [] }]);
  });

  it("labels elements", () => {
    expect(isNode(graphNode(1))).toBe(true);
    expect(isNode(graphLink(-1))).toBe(false);
    expect(elementLabel(graphNode(1))).toBe("node 1");
    expect(elementLabel(graphLink(-1))).toBe("edge -1");
  });

  it("provides node and edge specific actions", () => {
    setupQuery();
    const actions = createActions(ref("q1"));
    expect(actions.getActions(graphNode(1)).map((a) => a.key)).toEqual([
      "view",
      "edit",
      "expand",
      "add-edge",
      "delete",
    ]);
    expect(actions.getActions(graphLink(-3)).map((a) => a.key)).toEqual([
      "view",
      "edit",
      "delete",
    ]);
  });

  it("runs menu actions and closes the menu", () => {
    setupQuery();
    const actions = createActions(ref("q1"));
    const event = new Event("click");
    const run = (key: string, element: GraphNode | GraphLink) => {
      actions.openMenu(element, 1, 2);
      expect(actions.menu.value).toEqual({ element, x: 1, y: 2 });
      actions
        .getActions(element)
        .find((a) => a.key === key)
        ?.action?.({ event, params: undefined });
      expect(actions.menu.value).toBeUndefined();
    };

    run("view", graphNode(1));
    expect(actions.selectedElement.value?.id).toBe(1);
    expect(actions.panelMode.value).toBe("view");

    run("edit", graphLink(-3));
    expect(actions.selectedElement.value?.id).toBe(-3);
    expect(actions.panelMode.value).toBe("edit");

    run("add-edge", graphNode(1));
    expect(actions.pendingEdgeSourceId.value).toBe(1);

    run("delete", graphNode(2));
    expect(openModal).toHaveBeenCalled();

    run("expand", graphNode(1));
    expect(exec).toHaveBeenCalled();
  });

  it("dismisses menu, panel and add-edge mode", () => {
    setupQuery();
    const actions = createActions(ref("q1"));
    actions.viewElement(graphNode(1));
    actions.startAddEdge(graphNode(1));
    actions.openMenu(graphNode(1), 0, 0);
    actions.dismiss();
    expect(actions.selectedId.value).toBeUndefined();
    expect(actions.pendingEdgeSourceId.value).toBeUndefined();
    expect(actions.menu.value).toBeUndefined();
  });

  describe("delete", () => {
    it("confirms, deletes, patches results and re-runs exec queries", async () => {
      setupQuery();
      exec.mockResolvedValue([{ result: 1, elements: [dbNode(1)] }]);
      const actions = createActions(ref("q1"));
      actions.viewElement(graphNode(2));
      actions.startAddEdge(graphNode(2));

      actions.confirmDelete(graphNode(2));
      const modal = openModal.mock.calls[0]?.[0];
      expect(modal.header).toBe("Delete node 2");
      expect(JSON.stringify(modal.content)).toContain("edges");

      expect(await confirmModal()).toBe(true);
      expect(execMut).toHaveBeenCalledWith([{ Remove: { Ids: [{ Id: 2 }] } }]);
      expect(exec).toHaveBeenCalledWith([EXECUTED_QUERY]);
      expect(actions.selectedId.value).toBeUndefined();
      expect(actions.pendingEdgeSourceId.value).toBeUndefined();
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "success" }),
      );
      expect(actions.graphData.value.nodes.map((n) => n.id)).toEqual([1]);
    });

    it("does not mention edges when deleting an edge", () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      actions.confirmDelete(graphLink(-3));
      expect(
        JSON.stringify(openModal.mock.calls[0]?.[0].content),
      ).not.toContain("edges of the node");
    });

    it("never re-runs mutation queries but patches locally", async () => {
      setupQuery("exec_mut");
      const actions = createActions(ref("q1"));
      expect(await actions.deleteElement(graphNode(1))).toBe(true);
      expect(exec).not.toHaveBeenCalled();
      // Node 1 and its edge are gone.
      expect(actions.graphData.value.nodes.map((n) => n.id)).toEqual([2]);
      expect(actions.graphData.value.links).toEqual([]);
    });

    it("keeps selection of other elements and reports errors", async () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      actions.viewElement(graphNode(1));
      execMut.mockRejectedValueOnce(new Error("denied"));
      expect(await actions.deleteElement(graphNode(2))).toBe(false);
      expect(addNotification).toHaveBeenCalledWith({
        type: "error",
        title: "Delete failed",
        message: "denied",
      });
      expect(actions.selectedId.value).toBe(1);
    });

    it("does not delete locally when no database is open", async () => {
      const query = setupQuery();
      const actions = createActions(ref("q1"));
      execMut.mockResolvedValueOnce(undefined);
      expect(await actions.deleteElement(graphNode(2))).toBe(false);
      expect(query.value.results.exec?.[0]?.elements).toHaveLength(3);
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ title: "Delete failed" }),
      );
    });
  });

  describe("save values", () => {
    const values = [{ key: { String: "k" }, value: { String: "v" } }];

    it("writes changes, re-selects the element and returns to view mode", async () => {
      setupQuery();
      exec.mockResolvedValue([{ result: 1, elements: [dbNode(1, values)] }]);
      const actions = createActions(ref("q1"));
      actions.editElement(graphNode(1));

      expect(await actions.saveValues(graphNode(1), values)).toBe(true);
      expect(execMut).toHaveBeenCalledWith([
        expect.objectContaining({ InsertValues: expect.anything() }),
      ]);
      expect(exec).toHaveBeenCalledWith([
        { SelectValues: { keys: [], ids: { Ids: [{ Id: 1 }] } } },
      ]);
      expect(actions.panelMode.value).toBe("view");
      // The re-selected values become the element's stored values.
      expect(elementValues.set).toHaveBeenCalledWith(1, values);
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "success", title: "Values saved" }),
      );
    });

    it("reports a successful write even when reloading fails", async () => {
      setupQuery();
      exec.mockRejectedValueOnce(new Error("timeout"));
      const actions = createActions(ref("q1"));
      actions.editElement(graphNode(1));

      expect(await actions.saveValues(graphNode(1), values)).toBe(true);
      expect(actions.panelMode.value).toBe("view");
      const titles = addNotification.mock.calls.map(([n]) => n.title);
      expect(titles).toEqual(["Values saved", "Values not reloaded"]);
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          type: "warning",
          message: expect.stringContaining("timeout"),
        }),
      );
    });

    it("diffs against the values from when editing started", async () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      const started = [{ key: { String: "k" }, value: { I64: 1 } }];
      // The element now holds a value written by another client meanwhile.
      const current: GraphNode = {
        ...graphNode(1),
        rawValues: [{ key: { String: "k" }, value: { I64: 2 } }],
      };
      expect(await actions.saveValues(current, started, started)).toBe(true);
      expect(execMut).not.toHaveBeenCalled();
    });

    it("skips the mutation when nothing changed", async () => {
      setupQuery();
      exec.mockResolvedValue(undefined);
      const actions = createActions(ref("q1"));
      actions.editElement(graphNode(1));
      expect(await actions.saveValues(graphNode(1), [])).toBe(true);
      expect(execMut).not.toHaveBeenCalled();
      expect(actions.panelMode.value).toBe("view");
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "info", title: "No changes" }),
      );
    });

    it("does not report saving when no database is open", async () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      execMut.mockResolvedValueOnce(undefined);
      expect(
        await actions.saveValues(graphNode(1), [
          { key: { String: "k" }, value: { String: "v" } },
        ]),
      ).toBe(false);
      expect(addNotification).toHaveBeenCalledWith({
        type: "error",
        title: "Saving values failed",
        message: "No database is open, so nothing was changed.",
      });
    });

    it("reports errors (also non-Error values)", async () => {
      setupQuery();
      execMut.mockRejectedValueOnce("nope");
      const actions = createActions(ref("q1"));
      expect(await actions.saveValues(graphNode(1), values)).toBe(false);
      expect(addNotification).toHaveBeenCalledWith({
        type: "error",
        title: "Saving values failed",
        message: "nope",
      });
    });
  });

  describe("expand neighbours", () => {
    it("appends neighbours as expanded results", async () => {
      setupQuery();
      exec.mockResolvedValue([
        { result: 2, elements: [dbNode(1), dbNode(5), dbEdge(-6, 1, 5)] },
        { result: 1, elements: [dbNode(1)] },
      ]);
      const actions = createActions(ref("q1"));
      await actions.expandNeighbours(graphNode(1));
      expect(exec.mock.calls[0]?.[0]).toHaveLength(2);
      const nodes = actions.graphData.value.nodes;
      expect(nodes.find((n) => n.id === 5)?.source).toBe("context");
      // The expanded node itself is not counted.
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          message: "Loaded 1 node and 1 edge around node 1.",
        }),
      );
    });

    it("ignores missing results and reports errors", async () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      exec.mockResolvedValueOnce(undefined);
      await actions.expandNeighbours(graphNode(1));
      expect(addNotification).not.toHaveBeenCalled();

      exec.mockRejectedValueOnce(new Error("x"));
      await actions.expandNeighbours(graphNode(1));
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ title: "Loading neighbours failed" }),
      );
    });
  });

  describe("add edge", () => {
    it("picks the target on node click, confirms and opens the new edge", async () => {
      setupQuery();
      execMut.mockResolvedValue([{ result: 1, elements: [dbEdge(-9, 1, 2)] }]);
      const actions = createActions(ref("q1"));
      actions.openMenu(graphNode(1), 0, 0);
      actions.startAddEdge(graphNode(1));
      actions.clickNode(graphNode(2));
      expect(actions.menu.value).toBeUndefined();
      expect(openModal.mock.calls[0]?.[0].header).toBe("Add edge");

      expect(await confirmModal()).toBe(true);
      expect(execMut).toHaveBeenCalledWith([
        expect.objectContaining({
          InsertEdges: expect.objectContaining({
            from: { Ids: [{ Id: 1 }] },
            to: { Ids: [{ Id: 2 }] },
          }),
        }),
      ]);
      expect(actions.pendingEdgeSourceId.value).toBeUndefined();
      expect(actions.selectedId.value).toBe(-9);
      expect(actions.panelMode.value).toBe("edit");
    });

    it("handles missing results and errors", async () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      // No database open: nothing ran, so nothing is reported as created.
      actions.startAddEdge(graphNode(1));
      execMut.mockResolvedValueOnce(undefined);
      expect(await actions.addEdge(1, 2)).toBe(false);
      expect(actions.selectedId.value).toBeUndefined();
      expect(actions.pendingEdgeSourceId.value).toBe(1);
      expect(addNotification).toHaveBeenCalledWith({
        type: "error",
        title: "Creating edge failed",
        message: "No database is open, so nothing was changed.",
      });

      execMut.mockResolvedValueOnce([]);
      expect(await actions.addEdge(1, 2)).toBe(true);
      expect(actions.selectedId.value).toBeUndefined();

      execMut.mockRejectedValueOnce(new Error("bad"));
      expect(await actions.addEdge(1, 2)).toBe(false);
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ title: "Creating edge failed" }),
      );
    });

    it("does nothing when not in add-edge mode", () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      actions.completeAddEdge(graphNode(2));
      expect(openModal).not.toHaveBeenCalled();
      actions.clickNode(graphNode(2));
      expect(actions.selectedId.value).toBe(2);
    });
  });

  describe("refresh", () => {
    it("re-runs the queries of the last run, not the current builder rows", async () => {
      const query = setupQuery();
      // Edited in the builder but never run.
      query.value.rows.exec = [
        { id: "r", steps: [{ id: "x", type: "insert" }] },
      ];
      exec.mockResolvedValueOnce([{ result: 1, elements: [dbNode(9)] }]);
      await createActions(ref("q1")).refresh();
      expect(exec).toHaveBeenCalledTimes(1);
      expect(exec).toHaveBeenCalledWith([EXECUTED_QUERY]);
      expect(query.value.results.exec?.[0]?.elements[0]?.id).toBe(9);
    });

    it("re-runs the last context queries too and keeps expanded elements", async () => {
      const query = setupQuery();
      store.setQueryResults("q1", "context", [], {
        queries: [CONTEXT_QUERY],
      });
      store.appendExpandedResults("q1", [{ result: 1, elements: [dbNode(5)] }]);
      exec.mockImplementation(async ([q]) =>
        JSON.stringify(q) === JSON.stringify(CONTEXT_QUERY)
          ? [{ result: 1, elements: [dbNode(7)] }]
          : initialResults(),
      );
      await createActions(ref("q1")).refresh();
      expect(exec).toHaveBeenCalledTimes(2);
      expect(query.value.results.context?.[0]?.elements[0]?.id).toBe(7);
      expect(query.value.results.expanded?.[0]?.elements[0]?.id).toBe(5);
    });

    it("does nothing without stored queries", async () => {
      const query = setupQuery();
      query.value.executedQueries = undefined;
      await createActions(ref("q1")).refresh();
      expect(exec).not.toHaveBeenCalled();
    });

    it("skips missing results", async () => {
      const query = setupQuery();
      exec.mockResolvedValueOnce(undefined);
      await createActions(ref("q1")).refresh();
      expect(query.value.results.exec).toEqual(initialResults());
    });

    it("does not start while a run is in progress", async () => {
      const query = setupQuery();
      query.value.isRunning = true;
      await createActions(ref("q1")).refresh();
      expect(exec).not.toHaveBeenCalled();
    });

    it("drops its results when a run starts or finishes meanwhile", async () => {
      const query = setupQuery();
      const actions = createActions(ref("q1"));

      const started = deferred<unknown>();
      exec.mockReturnValueOnce(started.promise);
      const first = actions.refresh();
      query.value.isRunning = true;
      started.resolve([{ result: 1, elements: [dbNode(9)] }]);
      await first;
      expect(query.value.results.exec).toEqual(initialResults());

      query.value.isRunning = false;
      const finished = deferred<unknown>();
      exec.mockReturnValueOnce(finished.promise);
      const second = actions.refresh();
      query.value.lastRun = new Date(2);
      finished.resolve([{ result: 1, elements: [dbNode(9)] }]);
      await second;
      expect(query.value.results.exec).toEqual(initialResults());
    });

    it("keeps only the latest of overlapping refreshes", async () => {
      const query = setupQuery();
      const actions = createActions(ref("q1"));
      const older = deferred<unknown>();
      const newer = deferred<unknown>();
      exec
        .mockReturnValueOnce(older.promise)
        .mockReturnValueOnce(newer.promise);

      const first = actions.refresh();
      const second = actions.refresh();
      newer.resolve([{ result: 1, elements: [dbNode(8)] }]);
      await second;
      older.resolve([{ result: 1, elements: [dbNode(9)] }]);
      await first;
      expect(query.value.results.exec?.[0]?.elements[0]?.id).toBe(8);
    });

    it("tells the user when re-running fails and keeps patched results", async () => {
      const query = setupQuery();
      exec.mockRejectedValueOnce(new Error("invalid query"));
      await createActions(ref("q1")).refresh();
      expect(addNotification).toHaveBeenCalledWith({
        type: "warning",
        title: "Graph not refreshed",
        message: "Re-running the query failed: invalid query",
      });
      expect(query.value.results.exec).toEqual(initialResults());
    });

    it("stays quiet about failures of an outdated refresh", async () => {
      const query = setupQuery();
      const failing = deferred<unknown>();
      exec.mockReturnValueOnce(failing.promise);
      const pending = createActions(ref("q1")).refresh();
      query.value.lastRun = new Date(2);
      failing.reject(new Error("late"));
      await pending;
      expect(addNotification).not.toHaveBeenCalled();
    });
  });

  describe("stored values", () => {
    it("loads stored values of shown elements, up to the prefetch limit", () => {
      setupQuery();
      createActions(ref("q1"));
      expect(elementValues.load).toHaveBeenCalledWith([1, 2]);
    });

    it("starts over after a new run and loads again", async () => {
      const query = setupQuery();
      createActions(ref("q1"));
      expect(elementValues.clear).not.toHaveBeenCalled();
      query.value.lastRun = new Date(2);
      await nextTick();
      expect(elementValues.clear).toHaveBeenCalledTimes(1);
      expect(elementValues.load).toHaveBeenCalledTimes(2);
    });

    it("loads more when new elements are shown, without clearing", async () => {
      setupQuery();
      createActions(ref("q1"));
      store.appendExpandedResults("q1", [{ result: 1, elements: [dbNode(5)] }]);
      await nextTick();
      expect(elementValues.clear).not.toHaveBeenCalled();
      expect(elementValues.load).toHaveBeenLastCalledWith([1, 2]);
      expect(elementValues.load).toHaveBeenCalledTimes(2);
    });

    it("loads (or retries) the element opened in the panel", () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      actions.viewElement(graphNode(1));
      expect(elementValues.load).toHaveBeenLastCalledWith([1], { retry: true });
      actions.editElement(graphNode(2));
      expect(elementValues.load).toHaveBeenLastCalledWith([2], { retry: true });
      actions.loadValues(3);
      expect(elementValues.load).toHaveBeenLastCalledWith([3], { retry: true });
    });

    it("describes the selected element with its stored values", () => {
      setupQuery();
      const stored = [{ key: { String: "k" }, value: { String: "v" } }];
      elementValues.describe.mockImplementationOnce((element) => ({
        ...element,
        rawValues: stored,
      }));
      const actions = createActions(ref("q1"));
      expect(actions.selectedElement.value).toBeUndefined();
      actions.viewElement(graphNode(1));
      expect(actions.selectedElement.value?.rawValues).toEqual(stored);
      expect(actions.describeElement).toBe(elementValues.describe);
    });

    it("forgets the values of a deleted element", async () => {
      setupQuery();
      await createActions(ref("q1")).deleteElement(graphNode(2));
      expect(elementValues.remove).toHaveBeenCalledWith(2);
    });
  });

  describe("unsaved edits", () => {
    const editWithChanges = () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      actions.editElement(graphNode(1));
      actions.setUnsavedEdits(true);
      return actions;
    };
    const warned = () =>
      expect(addNotification).toHaveBeenCalledWith(
        expect.objectContaining({ type: "warning", title: "Unsaved changes" }),
      );

    it("keeps the panel open on a background click", () => {
      const actions = editWithChanges();
      actions.openMenu(graphNode(2), 0, 0);
      actions.dismiss();
      expect(actions.menu.value).toBeUndefined();
      expect(actions.selectedId.value).toBe(1);
      expect(actions.panelMode.value).toBe("edit");
      warned();
    });

    it("does not switch to another element", () => {
      const actions = editWithChanges();
      actions.viewElement(graphNode(2));
      actions.editElement(graphNode(2));
      expect(actions.selectedId.value).toBe(1);
      expect(actions.panelMode.value).toBe("edit");
      warned();
      // Editing the same element again is fine.
      actions.editElement(graphNode(1));
      expect(actions.panelMode.value).toBe("edit");
    });

    it("discards them on cancel or close", () => {
      const actions = editWithChanges();
      actions.cancelEdit();
      actions.viewElement(graphNode(2));
      expect(actions.selectedId.value).toBe(2);

      actions.editElement(graphNode(2));
      actions.setUnsavedEdits(true);
      actions.closePanel();
      actions.dismiss();
      actions.viewElement(graphNode(1));
      expect(actions.selectedId.value).toBe(1);
      expect(addNotification).not.toHaveBeenCalled();
    });

    it("does not block anything outside edit mode", () => {
      setupQuery();
      const actions = createActions(ref("q1"));
      actions.viewElement(graphNode(1));
      actions.setUnsavedEdits(true);
      actions.dismiss();
      expect(actions.selectedId.value).toBeUndefined();
    });
  });

  it("cancels editing without closing the panel", () => {
    setupQuery();
    const actions = createActions(ref("q1"));
    actions.editElement(graphNode(1));
    actions.cancelEdit();
    expect(actions.panelMode.value).toBe("view");
    expect(actions.selectedId.value).toBe(1);
  });

  it("does nothing without a query id", async () => {
    const actions = createActions(undefined);
    expect(await actions.saveValues(graphNode(1), [])).toBe(false);
    expect(await actions.deleteElement(graphNode(1))).toBe(false);
    expect(await actions.addEdge(1, 2)).toBe(false);
    await actions.expandNeighbours(graphNode(1));
    await actions.refresh();
    expect(exec).not.toHaveBeenCalled();
    expect(execMut).not.toHaveBeenCalled();
    expect(actions.graphData.value).toEqual({ nodes: [], links: [] });
  });

  it("provides actions to descendants", async () => {
    setupQuery();
    let provided: unknown;
    let injected: unknown;
    const Child = () => {
      injected = useGraphActions();
      return h("div");
    };
    const Parent = defineComponent({
      setup() {
        provided = provideGraphActions(ref("q1"));
        return () => h(Child);
      },
    });
    mount(Parent);
    await nextTick();
    expect(injected).toBe(provided);
  });
});
