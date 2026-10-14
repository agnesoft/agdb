import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QueryGraph from "./QueryGraph.vue";
import ForceGraph3D from "3d-force-graph";
import { nextTick, ref } from "vue";

const mockQueryRef = ref({
  id: "q1",
  isRunning: false,
  steps: { exec: [], exec_mut: [], context: [] },
  results: {},
  lastExecutedTab: undefined as "exec" | "exec_mut" | undefined,
});

vi.mock("../../composables/queryStore", () => ({
  useQueryStore: () => ({
    getQuery: (_id: string) => mockQueryRef,
  }),
}));

vi.mock("../../utils/graphDataUtils", () => ({
  convertElementsToGraphData: vi.fn(() => ({ nodes: [], links: [] })),
  mergeGraphData: vi.fn(
    (query: { nodes: unknown[]; links: unknown[] }) => query,
  ),
}));

vi.mock("3d-force-graph", () => {
  const instance = {
    graphData: vi.fn().mockReturnThis(),
    nodeColor: vi.fn().mockReturnThis(),
    linkColor: vi.fn().mockReturnThis(),
    nodeLabel: vi.fn().mockReturnThis(),
    linkLabel: vi.fn().mockReturnThis(),
    height: vi.fn().mockReturnThis(),
    width: vi.fn().mockReturnThis(),
    d3ReheatSimulation: vi.fn().mockReturnThis(),
    zoomToFit: vi.fn().mockReturnThis(),
    _destructor: vi.fn(),
  };

  const ForceGraph3D = vi.fn(function ForceGraph3DMock(_el: unknown) {
    return instance;
  });

  return {
    __esModule: true,
    default: ForceGraph3D,
  };
});

describe("QueryGraph", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockQueryRef.value = {
      id: "q1",
      isRunning: false,
      steps: { exec: [], exec_mut: [], context: [] },
      results: {},
      lastExecutedTab: undefined,
    };
  });

  const mountWithQueryId = (queryId = "q1") => {
    return mount(QueryGraph, {
      attachTo: document.body,
      global: {
        provide: {
          queryId: ref(queryId),
        },
      },
    });
  };

  it("renders the query graph container", () => {
    const wrapper = mountWithQueryId();
    expect(wrapper.find(".query-graph").exists()).toBe(true);
    wrapper.unmount();
  });

  it("initializes 3d-force-graph on mount with color callbacks", async () => {
    const wrapper = mountWithQueryId();
    await nextTick();

    expect(ForceGraph3D).toHaveBeenCalledTimes(1);
    const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;

    expect(instance?.graphData).toHaveBeenCalled();
    expect(instance?.nodeColor).toHaveBeenCalled();
    expect(instance?.linkColor).toHaveBeenCalled();
    expect(instance?.nodeLabel).toHaveBeenCalled();
    expect(instance?.linkLabel).toHaveBeenCalled();
    expect(instance?.height).toHaveBeenCalled();
    expect(instance?.width).toHaveBeenCalled();

    wrapper.unmount();
  });

  it("updates graph when query results change", async () => {
    vi.useFakeTimers();
    const wrapper = mountWithQueryId();
    await nextTick();

    const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;
    const initialCallCount = instance?.graphData.mock.calls.length ?? 0;

    const { convertElementsToGraphData } =
      await import("../../utils/graphDataUtils");
    vi.mocked(convertElementsToGraphData).mockReturnValue({
      nodes: [{ id: 1, values: [], source: "query" }],
      links: [],
    });

    mockQueryRef.value = {
      ...mockQueryRef.value,
      results: {
        exec: [
          { result: 1, elements: [{ id: 1, from: 0, to: 0, values: [] }] },
        ],
      },
      lastExecutedTab: "exec",
    };
    await nextTick();

    expect(instance?.graphData.mock.calls.length).toBeGreaterThan(
      initialCallCount,
    );
    expect(instance?.d3ReheatSimulation).toHaveBeenCalled();

    vi.advanceTimersByTime(300);
    expect(instance?.zoomToFit).toHaveBeenCalledWith(400);

    vi.useRealTimers();
    wrapper.unmount();
  });

  it("merges context results alongside query results", async () => {
    mockQueryRef.value = {
      ...mockQueryRef.value,
      results: {
        exec: [
          { result: 1, elements: [{ id: 1, from: 0, to: 0, values: [] }] },
        ],
        context: [
          { result: 2, elements: [{ id: 2, from: 0, to: 0, values: [] }] },
        ],
      },
      lastExecutedTab: "exec",
    };

    const wrapper = mountWithQueryId();
    await nextTick();

    const { mergeGraphData } = await import("../../utils/graphDataUtils");
    expect(mergeGraphData).toHaveBeenCalled();

    wrapper.unmount();
  });

  it("cancels pending zoom timer on rapid data updates", async () => {
    vi.useFakeTimers();
    const wrapper = mountWithQueryId();
    await nextTick();

    const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;
    const { convertElementsToGraphData } =
      await import("../../utils/graphDataUtils");
    vi.mocked(convertElementsToGraphData).mockReturnValue({
      nodes: [{ id: 1, values: [], source: "query" }],
      links: [],
    });

    // First update — starts a timer
    mockQueryRef.value = {
      ...mockQueryRef.value,
      results: {
        exec: [
          { result: 1, elements: [{ id: 1, from: 0, to: 0, values: [] }] },
        ],
      },
      lastExecutedTab: "exec",
    };
    await nextTick();

    // Second update before the 300ms timer fires — old timer should be cancelled
    mockQueryRef.value = {
      ...mockQueryRef.value,
      results: {
        exec: [
          { result: 2, elements: [{ id: 2, from: 0, to: 0, values: [] }] },
        ],
      },
      lastExecutedTab: "exec",
    };
    await nextTick();

    vi.advanceTimersByTime(300);
    // zoomToFit should only fire once (the second timer), not twice
    expect(instance?.zoomToFit).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
    wrapper.unmount();
  });

  it("clears zoom timer and calls _destructor on unmount", async () => {
    vi.useFakeTimers();
    const wrapper = mountWithQueryId();
    await nextTick();

    const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;
    const { convertElementsToGraphData } =
      await import("../../utils/graphDataUtils");
    vi.mocked(convertElementsToGraphData).mockReturnValue({
      nodes: [{ id: 1, values: [], source: "query" }],
      links: [],
    });

    // Trigger a data update to start the zoom timer
    mockQueryRef.value = {
      ...mockQueryRef.value,
      results: {
        exec: [
          { result: 1, elements: [{ id: 1, from: 0, to: 0, values: [] }] },
        ],
      },
      lastExecutedTab: "exec",
    };
    await nextTick();

    // Unmount before the 300ms timer fires
    wrapper.unmount();
    vi.advanceTimersByTime(300);

    // zoomToFit should NOT fire after unmount — timer was cleared
    expect(instance?.zoomToFit).not.toHaveBeenCalled();
    expect(instance?._destructor).toHaveBeenCalled();

    vi.useRealTimers();
  });
});
