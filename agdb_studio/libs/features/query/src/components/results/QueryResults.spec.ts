import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import { flushPromises, mount } from "@vue/test-utils";
import { defineComponent, h, nextTick, ref } from "vue";
import type { Components } from "@agnesoft/agdb_api";

const { exec, execMut, captured } = vi.hoisted(() => ({
  exec: vi.fn(),
  execMut: vi.fn(),
  captured: { actions: undefined as unknown },
}));

vi.mock("../../composables/dbExec", () => ({
  useDbExec: () => ({ exec, execMut, route: { params: {} } }),
}));

vi.mock("../graph/QueryGraph.vue", async () => {
  const { useGraphActions } = await import("../../composables/graphActions");
  return {
    default: defineComponent({
      setup() {
        captured.actions = useGraphActions();
        return () => h("div", { class: "mock-query-graph" });
      },
    }),
  };
});

import QueryResults from "./QueryResults.vue";
import { useQueryStore } from "../../composables/queryStore";
import type { GraphActions } from "../../composables/graphActions";
import type { GraphNode, QueryStep } from "../../composables/types";

const store = useQueryStore();
const el = (id: number, from = 0, to = 0) => ({ id, from, to, values: [] });

/** Stored values by element id, served to `select().ids()` queries. */
const db = new Map<number, Components.Schemas.DbKeyValue[]>();
const serveStoredValues = async (queries: Components.Schemas.QueryType[]) =>
  queries.map((query) => {
    const ids =
      "SelectValues" in query && "Ids" in query.SelectValues.ids
        ? query.SelectValues.ids.Ids.flatMap((id) =>
            "Id" in id ? [id.Id] : [],
          )
        : [];
    const elements = ids.flatMap((id) => {
      const values = db.get(id);
      return values ? [{ id, from: 0, to: 0, values }] : [];
    });
    return { result: elements.length, elements };
  });

const run = async (
  results: Components.Schemas.QueryResult[],
  steps: QueryStep["type"][] = ["select"],
  tab: "exec" | "exec_mut" = "exec",
) => {
  const query = store.getQuery("q1");
  if (!query) throw new Error("missing query");
  query.value.rows[tab] = [
    { id: "r", steps: steps.map((type) => ({ id: type, type })) },
  ];
  // Sets lastRun too, like a real run.
  store.setQueryResults("q1", tab, results, { stepTypes: steps });
  await nextTick();
};

const mountResults = () =>
  mount(QueryResults, {
    attachTo: document.body,
    global: { provide: { queryId: ref("q1") } },
  });

const actions = () => captured.actions as GraphActions;

describe("QueryResults", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    exec.mockReset();
    exec.mockResolvedValue(undefined);
    db.clear();
    store.clearQueries();
    store.addQuery({ id: "q1" });
    captured.actions = undefined;
    document.body.innerHTML = "";
  });

  it("shows the graph and a hint before the first run", () => {
    const wrapper = mountResults();
    expect(wrapper.find(".mock-query-graph").exists()).toBe(true);
    expect(wrapper.text()).toContain("Run a query to see results.");
    expect(wrapper.find("[data-testid=view-scalar]").exists()).toBe(false);
    wrapper.unmount();
  });

  it("shows a scalar card for node count", async () => {
    const wrapper = mountResults();
    await run([{ result: 42, elements: [] }], ["select", "select.node_count"]);
    expect(wrapper.find("[data-testid=result-scalar]").text()).toContain(
      "Node count",
    );
    expect(wrapper.find("[data-testid=result-scalar]").text()).toContain("42");
    expect(wrapper.find("[data-testid=view-scalar]").exists()).toBe(true);
    expect(wrapper.find("[data-testid=results-chip]").text()).toBe(
      "result: 42",
    );
    wrapper.unmount();
  });

  it("labels mutation results as affected", async () => {
    const wrapper = mountResults();
    await run([{ result: 2, elements: [] }], ["insert"], "exec_mut");
    expect(wrapper.find("[data-testid=result-scalar]").text()).toContain(
      "Affected",
    );
    wrapper.unmount();
  });

  it("uses the table for metadata and the graph for elements", async () => {
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }], ["select", "select.keys"]);
    expect(wrapper.find("[data-testid=results-table]").exists()).toBe(true);
    await run([{ result: 2, elements: [el(1), el(-2, 1, 1)] }]);
    expect(wrapper.find(".mock-query-graph").exists()).toBe(true);
    expect(wrapper.find("[data-testid=results-chip]").text()).toContain(
      "1 nodes · 1 edges",
    );
    wrapper.unmount();
  });

  it("keeps a chosen view until the result shape changes", async () => {
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }]);
    await wrapper.find("[data-testid=view-json]").trigger("click");
    expect(wrapper.find("[data-testid=results-json]").exists()).toBe(true);

    await run([{ result: 1, elements: [el(2)] }]);
    expect(wrapper.find("[data-testid=results-json]").exists()).toBe(true);

    await run([{ result: 0, elements: [] }]);
    expect(wrapper.find("[data-testid=result-scalar]").exists()).toBe(true);

    await wrapper.find("[data-testid=view-table]").trigger("click");
    expect(wrapper.find("[data-testid=results-table]").exists()).toBe(true);
    wrapper.unmount();
  });

  it("shows chips for batches, context, expanded elements and other ids", async () => {
    const wrapper = mountResults();
    store.setQueryResults("q1", "context", [{ result: 1, elements: [el(5)] }]);
    await run([
      { result: 1, elements: [el(0)] },
      { result: 1, elements: [el(1)] },
    ]);
    store.appendExpandedResults("q1", [{ result: 1, elements: [el(6)] }]);
    await nextTick();
    const chips = wrapper.findAll("[data-testid=results-chip]");
    expect(chips[0]?.text()).toContain("#1 · result: 1 · 1 other");
    expect(chips[1]?.text()).toContain("#2");
    expect(wrapper.text()).toContain("context: 1");
    expect(wrapper.text()).toContain("expanded: 1");
    wrapper.unmount();
  });

  it("opens the context menu and runs actions", async () => {
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }]);
    const node = actions().elementsById.value.get(1) as GraphNode;
    actions().openMenu(node, 10, 10);
    await nextTick();
    const menu = document.querySelector("[data-testid=graph-context-menu]");
    expect(menu?.textContent).toContain("node 1");
    document
      .querySelector<HTMLElement>("[data-testid=menu-item-view]")
      ?.click();
    await nextTick();
    expect(
      document.querySelector("[data-testid=graph-context-menu]"),
    ).toBeNull();
    expect(wrapper.find("[data-testid=element-details-panel]").exists()).toBe(
      true,
    );

    actions().openMenu(node, 10, 10);
    await nextTick();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    await nextTick();
    expect(actions().menu.value).toBeUndefined();
    wrapper.unmount();
  });

  it("opens the menu from table rows, also for elements not in the graph", async () => {
    const wrapper = mountResults();
    // A dangling edge (its nodes are not loaded) is not part of the graph.
    await run(
      [{ result: 2, elements: [el(1), el(-7, 8, 9)] }],
      ["select", "select.keys"],
    );
    const buttons = wrapper.findAll("[data-testid=results-row-menu]");
    await buttons[0]?.trigger("click", { clientX: 1, clientY: 2 });
    expect(actions().menu.value?.element.id).toBe(1);
    await buttons[1]?.trigger("click", { clientX: 1, clientY: 2 });
    expect(actions().menu.value?.element.id).toBe(-7);
    wrapper.unmount();
  });

  it("edits, cancels, saves and closes the details panel", async () => {
    db.set(1, []);
    exec.mockImplementation(serveStoredValues);
    execMut.mockResolvedValue([]);
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }], ["select", "select.keys"]);
    const node = actions().elementsById.value.get(1) as GraphNode;
    actions().viewElement(node);
    await flushPromises();

    await wrapper.find("[data-testid=panel-edit]").trigger("click");
    expect(actions().panelMode.value).toBe("edit");
    await wrapper.find("[data-testid=panel-cancel]").trigger("click");
    expect(actions().panelMode.value).toBe("view");

    await wrapper.find("[data-testid=panel-edit]").trigger("click");
    await wrapper.find("[data-testid=panel-add-row]").trigger("click");
    await wrapper.find("[data-testid=panel-key-input]").setValue("k");
    await wrapper.find("form").trigger("submit");
    await flushPromises();
    expect(execMut).toHaveBeenCalled();
    expect(actions().panelMode.value).toBe("view");

    await wrapper.find("[data-testid=panel-close]").trigger("click");
    expect(wrapper.find("[data-testid=element-details-panel]").exists()).toBe(
      false,
    );
    wrapper.unmount();
  });

  it("shows all stored values, not only those the query returned", async () => {
    db.set(1, [{ key: { String: "name" }, value: { String: "Alice" } }]);
    exec.mockImplementation(serveStoredValues);
    const wrapper = mountResults();
    // e.g. select().edge_count(): only a computed value comes back.
    await run([
      {
        result: 1,
        elements: [
          {
            id: 1,
            from: 0,
            to: 0,
            values: [{ key: { String: "edge_count" }, value: { U64: 0 } }],
          },
        ],
      },
    ]);
    await flushPromises();
    const node = actions().elementsById.value.get(1);
    if (!node) throw new Error("missing node");
    actions().viewElement(node);
    await nextTick();

    const stored = wrapper.findAll("[data-testid=panel-value-row]");
    expect(stored.map((row) => row.text())).toEqual(["nameAlicestring"]);
    const fromQuery = wrapper.findAll("[data-testid=panel-query-value-row]");
    expect(fromQuery.map((row) => row.text())).toEqual(["edge_count0u64"]);
    // Hover tooltips get the same data.
    expect(actions().describeElement(node).values).toEqual([
      { key: "name", value: "Alice" },
    ]);
    wrapper.unmount();
  });

  it("retries loading stored values from the panel", async () => {
    exec.mockRejectedValueOnce(new Error("offline"));
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }]);
    await flushPromises();
    const node = actions().elementsById.value.get(1);
    if (!node) throw new Error("missing node");
    // Opening the panel retries once; let that fail too.
    exec.mockRejectedValueOnce(new Error("offline"));
    actions().viewElement(node);
    await flushPromises();
    expect(wrapper.find("[data-testid=panel-values-error]").exists()).toBe(
      true,
    );
    expect(
      wrapper.find<HTMLButtonElement>("[data-testid=panel-edit]").element
        .disabled,
    ).toBe(true);

    db.set(1, []);
    exec.mockImplementation(serveStoredValues);
    await wrapper.find("[data-testid=panel-values-retry]").trigger("click");
    await flushPromises();
    expect(wrapper.find("[data-testid=panel-values-error]").exists()).toBe(
      false,
    );
    wrapper.unmount();
  });

  it("does not write back values another client changed while editing", async () => {
    const kv = (value: number) => ({
      key: { String: "k" },
      value: { I64: value },
    });
    db.set(1, [kv(1)]);
    exec.mockImplementation(serveStoredValues);
    execMut.mockResolvedValue([]);
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }]);
    const node = actions().elementsById.value.get(1);
    if (!node) throw new Error("missing node");
    actions().editElement(node);
    await flushPromises();
    await wrapper.find("[data-testid=panel-add-row]").trigger("click");
    const keyInputs = wrapper.findAll("[data-testid=panel-key-input]");
    await keyInputs[1]?.setValue("other");

    // Another client changes "k"; a new run reloads the stored values.
    db.set(1, [kv(2)]);
    const query = store.getQuery("q1");
    if (!query) throw new Error("missing query");
    query.value.lastRun = new Date(Date.now() + 1000);
    await flushPromises();
    // The edits in progress survive the reload.
    expect(
      wrapper.findAll<HTMLInputElement>("[data-testid=panel-key-input]")[1]
        ?.element.value,
    ).toBe("other");

    await wrapper.find("form").trigger("submit");
    await flushPromises();

    // Only the new row is inserted; "k" is neither removed nor overwritten.
    expect(execMut).toHaveBeenCalledTimes(1);
    const sent = JSON.stringify(execMut.mock.calls[0]?.[0]);
    expect(sent).toContain("other");
    expect(sent).not.toContain('"I64":1');
    expect(sent).not.toContain("RemoveValues");
    wrapper.unmount();
  });

  it("shows the add-edge banner and cancels it", async () => {
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }]);
    const node = actions().elementsById.value.get(1) as GraphNode;

    actions().startAddEdge(node);
    await nextTick();
    expect(wrapper.find("[data-testid=add-edge-banner]").text()).toContain(
      "node 1",
    );
    await wrapper.find("[data-testid=add-edge-cancel]").trigger("click");
    expect(wrapper.find("[data-testid=add-edge-banner]").exists()).toBe(false);

    actions().startAddEdge(node);
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "a" }));
    expect(actions().pendingEdgeSourceId.value).toBe(1);
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(actions().pendingEdgeSourceId.value).toBeUndefined();
    // Escape outside add-edge mode is a no-op.
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    wrapper.unmount();
  });

  it("labels results by the steps that ran, not later builder edits", async () => {
    const wrapper = mountResults();
    await run([{ result: 42, elements: [] }], ["select", "select.node_count"]);
    const query = store.getQuery("q1");
    if (!query) throw new Error("missing query");
    query.value.rows.exec = [{ id: "r", steps: [{ id: "s", type: "select" }] }];
    await nextTick();
    expect(wrapper.find("[data-testid=result-scalar]").text()).toContain(
      "Node count",
    );
    wrapper.unmount();
  });

  it("loads stored values once per run, without stale values in between", async () => {
    db.set(1, [{ key: { String: "k" }, value: { I64: 1 } }]);
    exec.mockImplementation(serveStoredValues);
    const wrapper = mountResults();
    await run([{ result: 1, elements: [el(1)] }]);
    await flushPromises();
    expect(exec).toHaveBeenCalledTimes(1);

    // A mutation run changes the value; the next run must not show the old one.
    db.set(1, [{ key: { String: "k" }, value: { I64: 2 } }]);
    let release = () => {};
    exec.mockImplementationOnce(
      (queries: Components.Schemas.QueryType[]) =>
        new Promise((resolve) => {
          release = () => resolve(serveStoredValues(queries));
        }),
    );
    await run([{ result: 1, elements: [el(1)] }]);
    const node = actions().elementsById.value.get(1);
    if (!node) throw new Error("missing node");
    expect(actions().describeElement(node).valuesState).toBe("loading");
    release();
    await flushPromises();
    expect(exec).toHaveBeenCalledTimes(2);
    expect(actions().describeElement(node).values).toEqual([
      { key: "k", value: "2" },
    ]);
    wrapper.unmount();
  });

  it("renders without a query id", () => {
    const wrapper = mount(QueryResults);
    expect(wrapper.text()).toContain("Run a query to see results.");
    wrapper.unmount();
  });
});
