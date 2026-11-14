import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import { ref } from "vue";
import { useGraphData } from "./graphData";
import { useQueryStore } from "./queryStore";

const store = useQueryStore();

const node = (id: number) => ({ id, from: 0, to: 0, values: [] });
const edge = (id: number, from: number, to: number) => ({
  id,
  from,
  to,
  values: [],
});

describe("useGraphData", () => {
  beforeEach(() => {
    store.clearQueries();
    store.addQuery({ id: "q1" });
  });

  it("is empty without a query id, an unknown query or results", () => {
    expect(useGraphData(undefined).graphData.value).toEqual({
      nodes: [],
      links: [],
    });
    expect(useGraphData(ref(undefined)).query.value).toBeUndefined();
    expect(useGraphData(ref("missing")).graphData.value).toEqual({
      nodes: [],
      links: [],
    });
    expect(useGraphData(ref("q1")).graphData.value).toEqual({
      nodes: [],
      links: [],
    });
  });

  it("shows the results of the last executed tab as query elements", () => {
    store.setQueryResults("q1", "exec", [{ result: 1, elements: [node(1)] }]);
    store.setQueryResults("q1", "exec_mut", [
      { result: 2, elements: [node(2), node(3)] },
    ]);
    const { graphData } = useGraphData(ref("q1"));
    expect(graphData.value.nodes.map((n) => [n.id, n.source])).toEqual([
      [2, "query"],
      [3, "query"],
    ]);
  });

  it("falls back to exec results before any run", () => {
    const query = store.getQuery("q1");
    if (!query) throw new Error("missing query");
    query.value.results = { exec: [{ result: 1, elements: [node(1)] }] };
    expect(
      useGraphData(ref("q1")).graphData.value.nodes.map((n) => n.id),
    ).toEqual([1]);
  });

  it("merges context and expanded elements as context", () => {
    store.setQueryResults("q1", "exec", [
      { result: 2, elements: [node(1), node(2), edge(-3, 1, 2)] },
    ]);
    store.setQueryResults("q1", "context", [
      { result: 1, elements: [node(4)] },
    ]);
    store.appendExpandedResults("q1", [
      // Node 1 is already a query element and stays one.
      { result: 2, elements: [node(1), node(5), edge(-6, 1, 5)] },
    ]);
    const { graphData } = useGraphData(ref("q1"));
    const sources = Object.fromEntries(
      graphData.value.nodes.map((n) => [n.id, n.source]),
    );
    expect(sources).toEqual({
      1: "query",
      2: "query",
      4: "context",
      5: "context",
    });
    expect(
      graphData.value.links.map((l) => l.id).sort((a, b) => a - b),
    ).toEqual([-6, -3]);
  });

  it("hides dangling links in the graph but still looks them up by id", () => {
    store.setQueryResults("q1", "exec", [
      { result: 2, elements: [node(1), edge(-2, 1, 9)] },
    ]);
    const { graphData, elementsById } = useGraphData(ref("q1"));
    expect(graphData.value.links).toEqual([]);
    expect(elementsById.value.get(-2)?.id).toBe(-2);
    expect(elementsById.value.get(1)?.id).toBe(1);
  });

  it("updates when the results change", () => {
    const { graphData } = useGraphData(ref("q1"));
    expect(graphData.value.nodes).toHaveLength(0);
    store.setQueryResults("q1", "exec", [{ result: 1, elements: [node(7)] }]);
    expect(graphData.value.nodes.map((n) => n.id)).toEqual([7]);
  });
});
