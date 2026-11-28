import {
  afterEach,
  describe,
  it,
  expect,
  vi,
  beforeEach,
  afterAll,
} from "vitest";
import { mount } from "@vue/test-utils";
import QueryGraph from "./QueryGraph.vue";
import type { ColorRef } from "../../utils/graphColorRules";
import ForceGraph3D from "3d-force-graph";
import { nextTick, reactive, ref } from "vue";
import {
  GRAPH_ACTIONS_KEY,
  createGraphActions,
} from "../../composables/graphActions";

const mockQueryRef = ref({
  id: "q1",
  isRunning: false,
  rows: { exec: [], exec_mut: [], context: [] },
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
  dedupeResultElements: vi.fn(() => []),
  dbValueToString: vi.fn(String),
  toDisplayValues: vi.fn(() => []),
  mergeGraphData: vi.fn(
    (query: { nodes: unknown[]; links: unknown[] }) => query,
  ),
  filterDanglingLinks: vi.fn(
    (data: { nodes: unknown[]; links: unknown[] }) => data,
  ),
}));

vi.mock("3d-force-graph", () => {
  const METHODS = [
    "graphData",
    "backgroundColor",
    "nodeRelSize",
    "nodeOpacity",
    "nodeColor",
    "nodeLabel",
    "linkOpacity",
    "linkMaterial",
    "linkColor",
    "linkWidth",
    "linkCurvature",
    "linkDirectionalArrowLength",
    "linkDirectionalArrowRelPos",
    "linkDirectionalArrowColor",
    "linkDirectionalParticles",
    "linkDirectionalParticleWidth",
    "linkLabel",
    "onNodeHover",
    "onLinkHover",
    "onNodeClick",
    "onNodeRightClick",
    "onLinkClick",
    "onLinkRightClick",
    "onBackgroundClick",
    "onBackgroundRightClick",
    "onEngineStop",
    "height",
    "width",
    "d3ReheatSimulation",
    "zoomToFit",
    "emitParticle",
  ];
  const instance: Record<string, ReturnType<typeof vi.fn>> = {
    _destructor: vi.fn(),
    // Projects graph coordinates 1:1 to the screen.
    graph2ScreenCoords: vi.fn((x: number, y: number) => ({ x, y })),
  };
  for (const method of METHODS) {
    instance[method] = vi.fn().mockReturnValue(instance);
  }

  const ForceGraph3D = vi.fn(function ForceGraph3DMock(_el: unknown) {
    return instance;
  });

  return {
    __esModule: true,
    default: ForceGraph3D,
  };
});

// jsdom has no ResizeObserver; capture the callback so tests can trigger resizes.
let resizeCallback: ResizeObserverCallback | undefined;
const resizeObserverDisconnect = vi.fn();
vi.stubGlobal(
  "ResizeObserver",
  class {
    constructor(callback: ResizeObserverCallback) {
      resizeCallback = callback;
    }
    observe = vi.fn();
    disconnect = resizeObserverDisconnect;
  },
);

describe("QueryGraph", () => {
  afterAll(() => {
    vi.unstubAllGlobals();
  });

  beforeEach(() => {
    vi.clearAllMocks();
    resizeCallback = undefined;
    mockQueryRef.value = {
      id: "q1",
      isRunning: false,
      rows: { exec: [], exec_mut: [], context: [] },
      results: {},
      lastExecutedTab: undefined,
    };
  });

  /** Mounts the graph with real graph actions backed by the mocked store. */
  const mountWithQueryId = (queryId = "q1") => {
    return mount(QueryGraph, {
      attachTo: document.body,
      global: {
        provide: {
          [GRAPH_ACTIONS_KEY as symbol]: createGraphActions(ref(queryId)),
        },
      },
    });
  };

  it("fails loudly when rendered without graph actions", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(() => mount(QueryGraph)).toThrow(
      "QueryGraph must be rendered inside QueryResults",
    );
    warn.mockRestore();
  });

  it("renders the query graph container", async () => {
    const wrapper = mountWithQueryId();
    expect(wrapper.find(".query-graph").exists()).toBe(true);
    const event = new MouseEvent("contextmenu", { cancelable: true });
    wrapper.find(".query-graph").element.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
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

  it("resizes the graph with its container and stops observing on unmount", async () => {
    const wrapper = mountWithQueryId();
    await nextTick();
    const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;

    resizeCallback?.(
      [{ contentRect: { width: 640, height: 320 } } as ResizeObserverEntry],
      {} as ResizeObserver,
    );

    expect(instance?.height).toHaveBeenLastCalledWith(320);
    expect(instance?.width).toHaveBeenLastCalledWith(640);

    wrapper.unmount();
    expect(resizeObserverDisconnect).toHaveBeenCalled();
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
      nodes: [{ id: 1, values: [], rawValues: [], source: "query" }],
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

  it("filters dangling links from graph data", async () => {
    mockQueryRef.value = {
      ...mockQueryRef.value,
      results: {
        exec: [
          { result: 1, elements: [{ id: 1, from: 0, to: 0, values: [] }] },
        ],
      },
      lastExecutedTab: "exec",
    };

    const wrapper = mountWithQueryId();
    await nextTick();

    const { filterDanglingLinks } = await import("../../utils/graphDataUtils");
    expect(filterDanglingLinks).toHaveBeenCalled();

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
      nodes: [{ id: 1, values: [], rawValues: [], source: "query" }],
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
      nodes: [{ id: 1, values: [], rawValues: [], source: "query" }],
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

  describe("interactions", () => {
    const node = { id: 1, values: [], rawValues: [], source: "query" };
    const link = {
      id: -2,
      values: [],
      rawValues: [],
      source: 1,
      target: 1,
      elementSource: "query",
    };

    const makeActions = () => ({
      offsets: reactive(new Map<number, { x: number; y: number }>()),
      cardOffset(id: number) {
        return this.offsets.get(id);
      },
      setCardOffset(id: number, offset: { x: number; y: number }) {
        this.offsets.set(id, offset);
      },
      graphData: ref({ nodes: [node], links: [link] }),
      selectedId: ref<number>(),
      pendingEdgeSourceId: ref<number>(),
      pinnedIds: ref<number[]>([]),
      pinnedElements: ref<object[]>([]),
      lastAddedEdgeId: ref<number>(),
      togglePin: vi.fn(),
      unpin: vi.fn(),
      paletteSize: ref(5),
      colorResolver: ref({}),
      colorOf: vi.fn((): ColorRef | undefined => undefined),
      clickNode: vi.fn(),
      openMenu: vi.fn(),
      closeMenu: vi.fn(),
      viewElement: vi.fn(),
      dismiss: vi.fn(),
      // Adds a stored value, as the stored-values cache would.
      describeElement: vi.fn(<T extends object>(element: T) => ({
        ...element,
        values: [{ key: "stored", value: "yes" }],
      })),
    });

    const mountWithActions = (actions = makeActions()) => {
      const wrapper = mount(QueryGraph, {
        attachTo: document.body,
        global: {
          provide: {
            [GRAPH_ACTIONS_KEY as symbol]: actions,
          },
        },
      });
      const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;
      const handler = (name: string) =>
        instance?.[name].mock.calls[0]?.[0] as (...args: unknown[]) => void;
      return { wrapper, actions, instance, handler };
    };

    it("wires click and context menu handlers to graph actions", () => {
      const { wrapper, actions, handler } = mountWithActions();
      const event = { clientX: 5, clientY: 6, preventDefault: vi.fn() };

      handler("onNodeClick")(node);
      expect(actions.clickNode).toHaveBeenCalledWith(node, { pin: false });
      handler("onNodeClick")(node, { shiftKey: true });
      expect(actions.clickNode).toHaveBeenCalledWith(node, { pin: true });

      handler("onNodeRightClick")(node, event);
      expect(event.preventDefault).toHaveBeenCalled();
      expect(actions.openMenu).toHaveBeenCalledWith(node, 5, 6);

      handler("onLinkClick")(link);
      expect(actions.closeMenu).toHaveBeenCalled();
      expect(actions.viewElement).toHaveBeenCalledWith(link);
      handler("onLinkClick")(link, { shiftKey: true });
      expect(actions.togglePin).toHaveBeenCalledWith(link);
      expect(actions.viewElement).toHaveBeenCalledTimes(1);

      handler("onLinkRightClick")(link, { clientX: 1, clientY: 2 });
      expect(actions.openMenu).toHaveBeenCalledWith(link, 1, 2);

      handler("onBackgroundClick")();
      expect(actions.dismiss).toHaveBeenCalled();

      actions.closeMenu.mockClear();
      handler("onBackgroundRightClick")();
      expect(actions.closeMenu).toHaveBeenCalled();
      wrapper.unmount();
    });

    it("restyles on hover and shows a pointer cursor", async () => {
      const { wrapper, instance, handler } = mountWithActions();
      const calls = () => instance?.nodeColor.mock.calls.length ?? 0;
      const before = calls();

      handler("onNodeHover")(node);
      await nextTick();
      expect(calls()).toBeGreaterThan(before);
      expect(wrapper.find(".query-graph").classes()).toContain("hovering");

      handler("onNodeHover")(null);
      await nextTick();
      expect(wrapper.find(".query-graph").classes()).not.toContain("hovering");

      handler("onLinkHover")(link);
      await nextTick();
      expect(wrapper.find(".query-graph").classes()).toContain("hovering");
      handler("onLinkHover")(null);
      wrapper.unmount();

      // Late hover events after unmount do not touch the destroyed graph.
      const callsAfterUnmount = calls();
      handler("onNodeHover")(node);
      expect(calls()).toBe(callsAfterUnmount);
    });

    it("restyles when the selection or pending edge source changes", async () => {
      const { wrapper, actions, instance } = mountWithActions();
      const before = instance?.linkColor.mock.calls.length ?? 0;
      actions.selectedId.value = 1;
      await nextTick();
      actions.pendingEdgeSourceId.value = 1;
      await nextTick();
      expect(instance?.linkColor.mock.calls.length).toBe(before + 2);
      wrapper.unmount();
    });

    describe("overlay", () => {
      let frames: FrameRequestCallback[] = [];
      const runFrame = async () => {
        const pending = frames;
        frames = [];
        for (const cb of pending) cb(0);
        await nextTick();
      };
      beforeEach(() => {
        frames = [];
        vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => {
          frames.push(cb);
          return frames.length;
        });
        vi.stubGlobal("cancelAnimationFrame", vi.fn());
      });
      afterEach(() => vi.unstubAllGlobals());

      // jsdom has no PointerEvent; a MouseEvent carries the same fields.
      const pointer = (type: string, init: MouseEventInit = {}) =>
        new MouseEvent(type, { bubbles: true, ...init });

      const laidOut = () => {
        const a = { ...node, x: 10, y: 20, z: 0 };
        const b = { ...node, id: 2, x: 50, y: 60, z: 0 };
        // As the library leaves them: endpoints replaced by node objects.
        const edge = { ...link, id: -3, source: a, target: b };
        const actions = makeActions();
        actions.graphData.value = {
          nodes: [a, b],
          links: [edge as unknown as typeof link],
        };
        return { actions, a, b, edge };
      };

      it("shows pinned cards next to their element and follows it", async () => {
        const { actions, a, edge } = laidOut();
        const { wrapper } = mountWithActions(actions);
        actions.pinnedIds.value = [1, -3];
        actions.pinnedElements.value = [
          { ...a, alias: "alice" },
          { ...edge, values: [{ key: "w", value: "5" }] },
        ];
        await nextTick();
        await runFrame();
        const cards = wrapper.findAll("[data-testid=pinned-card]");
        expect(cards).toHaveLength(2);
        expect(cards[0]?.text()).toContain("Node 1");
        expect(cards[0]?.text()).toContain("alice");
        expect(cards[0]?.attributes("style")).toContain("translate(28px, 2px)");
        // An edge's card sits at its middle.
        expect(cards[1]?.attributes("style")).toContain(
          "translate(48px, 22px)",
        );
        expect(cards[1]?.text()).toContain("1 → 2");
        expect(wrapper.findAll(".leader-line")).toHaveLength(2);

        a.x = 100;
        await runFrame();
        expect(
          wrapper.find("[data-testid=pinned-card]").attributes("style"),
        ).toContain("translate(118px, 2px)");

        await wrapper.find("[data-testid=pinned-card-unpin]").trigger("click");
        expect(actions.unpin).toHaveBeenCalledWith(1);
        await wrapper.find(".card-header").trigger("dblclick");
        expect(actions.viewElement).toHaveBeenCalled();
        wrapper.unmount();
      });

      it("moves a card by dragging its header", async () => {
        const { actions, a } = laidOut();
        const { wrapper } = mountWithActions(actions);
        actions.pinnedIds.value = [1];
        actions.pinnedElements.value = [a];
        await nextTick();
        await runFrame();
        const header = wrapper.find(".card-header");
        header.element.dispatchEvent(
          pointer("pointerdown", { button: 0, clientX: 0, clientY: 0 }),
        );
        window.dispatchEvent(
          pointer("pointermove", { clientX: 30, clientY: 40 }),
        );
        window.dispatchEvent(new Event("pointerup"));
        // Further moves no longer drag it.
        window.dispatchEvent(
          pointer("pointermove", { clientX: 99, clientY: 99 }),
        );
        await nextTick();
        expect(
          wrapper.find("[data-testid=pinned-card]").attributes("style"),
        ).toContain("translate(58px, 42px)");
        // The offset is kept with the pin.
        expect(actions.offsets.get(1)).toEqual({ x: 48, y: 22 });
        wrapper.unmount();
      });

      it("ends a drag on cancel and on unmount", async () => {
        const { actions, a } = laidOut();
        const { wrapper } = mountWithActions(actions);
        actions.pinnedIds.value = [1];
        actions.pinnedElements.value = [a];
        await nextTick();
        await runFrame();
        const header = wrapper.find(".card-header");
        const down = () =>
          header.element.dispatchEvent(
            pointer("pointerdown", { button: 0, clientX: 0, clientY: 0 }),
          );
        down();
        window.dispatchEvent(new Event("pointercancel"));
        window.dispatchEvent(
          pointer("pointermove", { clientX: 30, clientY: 40 }),
        );
        await nextTick();
        expect(
          wrapper.find("[data-testid=pinned-card]").attributes("style"),
        ).toContain("translate(28px, 2px)");
        down();
        // A second drag replaces the first.
        down();
        const remove = vi.spyOn(window, "removeEventListener");
        wrapper.unmount();
        expect(remove).toHaveBeenCalledWith(
          "pointermove",
          expect.any(Function),
        );
        remove.mockRestore();
      });

      it("hides cards of elements without a position yet", async () => {
        const actions = makeActions();
        const { wrapper } = mountWithActions(actions);
        actions.pinnedIds.value = [1, -2, 9];
        actions.pinnedElements.value = [node];
        await nextTick();
        await runFrame();
        expect(wrapper.find("[data-testid=pinned-card]").exists()).toBe(false);
        // Nothing attached any more: tracking stops.
        actions.pinnedIds.value = [];
        await nextTick();
        await runFrame();
        expect(frames).toHaveLength(0);
        wrapper.unmount();
      });

      it("draws a line from the add-edge source to the pointer, snapping to a target", async () => {
        const { actions, b } = laidOut();
        const { wrapper, handler } = mountWithActions(actions);
        actions.pendingEdgeSourceId.value = 1;
        await nextTick();
        await runFrame();
        // No pointer yet.
        expect(wrapper.find("[data-testid=graph-rubber-band]").exists()).toBe(
          false,
        );
        wrapper
          .find(".query-graph-wrapper")
          .element.dispatchEvent(
            pointer("pointermove", { clientX: 70, clientY: 80 }),
          );
        await runFrame();
        const line = wrapper.find("[data-testid=graph-rubber-band]");
        expect(line.attributes()).toMatchObject({
          x1: "10",
          y1: "20",
          x2: "70",
          y2: "80",
        });
        expect(line.classes()).not.toContain("snapped");
        expect(wrapper.find(".query-graph").classes()).toContain("adding-edge");

        handler("onNodeHover")(b);
        await runFrame();
        expect(
          wrapper.find("[data-testid=graph-rubber-band]").attributes(),
        ).toMatchObject({ x2: "50", y2: "60" });
        expect(
          wrapper.find("[data-testid=graph-rubber-band]").classes(),
        ).toContain("snapped");

        handler("onNodeHover")(null);
        wrapper
          .find(".query-graph-wrapper")
          .element.dispatchEvent(pointer("pointerleave"));
        await runFrame();
        expect(wrapper.find("[data-testid=graph-rubber-band]").exists()).toBe(
          false,
        );
        wrapper.unmount();
      });

      it("animates a new edge once it is in the graph", async () => {
        vi.useFakeTimers();
        const { actions, edge, a, b } = laidOut();
        const { wrapper, instance } = mountWithActions(actions);
        // Created, but the refreshed data is not there yet.
        actions.lastAddedEdgeId.value = -7;
        await nextTick();
        expect(instance?.emitParticle).not.toHaveBeenCalled();
        const created = { ...edge, id: -7 };
        actions.graphData.value = {
          nodes: [a, b],
          links: [created as unknown as typeof link],
        };
        await nextTick();
        vi.runAllTimers();
        expect(instance?.emitParticle).toHaveBeenCalledTimes(3);

        // An edge already shown is animated right away.
        actions.lastAddedEdgeId.value = undefined;
        await nextTick();
        actions.lastAddedEdgeId.value = -7;
        await nextTick();
        vi.runAllTimers();
        expect(instance?.emitParticle).toHaveBeenCalledTimes(6);
        vi.useRealTimers();
        wrapper.unmount();
      });
    });

    it("passes tooltips for nodes and links", () => {
      const { wrapper, handler } = mountWithActions();
      expect(handler("nodeLabel")(node)).toContain("Node 1");
      expect(handler("linkLabel")(link)).toContain("Edge -2");
      // Tooltips show the stored values, not only what the query returned.
      expect(handler("nodeLabel")(node)).toContain("stored");
      expect(handler("linkLabel")(link)).toContain("stored");
      wrapper.unmount();
    });

    it("re-reads the theme when the color scheme changes", () => {
      const listeners: (() => void)[] = [];
      const removeEventListener = vi.fn();
      const matchMedia = vi.fn(() => ({
        addEventListener: (_: string, cb: () => void) => listeners.push(cb),
        removeEventListener,
      }));
      vi.stubGlobal("matchMedia", matchMedia);

      const { wrapper, instance } = mountWithActions();
      const before = instance?.backgroundColor.mock.calls.length ?? 0;
      listeners.forEach((cb) => cb());
      expect(instance?.backgroundColor.mock.calls.length).toBe(before + 1);

      wrapper.unmount();
      expect(removeEventListener).toHaveBeenCalledWith(
        "change",
        expect.any(Function),
      );
      // Applying the theme after unmount is a no-op.
      listeners.forEach((cb) => cb());
      expect(instance?.backgroundColor.mock.calls.length).toBe(before + 1);
      vi.unstubAllGlobals();
    });
  });

  describe("sizing and remount", () => {
    it("follows container size changes and disconnects on unmount", async () => {
      let callback: ResizeObserverCallback | undefined;
      const observe = vi.fn();
      const disconnect = vi.fn();
      vi.stubGlobal(
        "ResizeObserver",
        vi.fn(function (cb: ResizeObserverCallback) {
          callback = cb;
          return { observe, disconnect };
        }),
      );
      const wrapper = mountWithQueryId();
      const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;
      expect(observe).toHaveBeenCalled();

      const resize = (width: number, height: number) =>
        callback?.(
          [{ contentRect: { width, height } } as ResizeObserverEntry],
          {} as ResizeObserver,
        );
      resize(800, 600);
      expect(instance?.width).toHaveBeenLastCalledWith(800);
      expect(instance?.height).toHaveBeenLastCalledWith(600);

      instance?.width.mockClear();
      resize(0, 0);
      callback?.([], {} as ResizeObserver);
      expect(instance?.width).not.toHaveBeenCalled();

      wrapper.unmount();
      expect(disconnect).toHaveBeenCalled();
      // Resizes after unmount are ignored.
      resize(10, 10);
      expect(instance?.width).not.toHaveBeenCalled();
      vi.unstubAllGlobals();
    });

    it("zooms to fit on mount when there is data", async () => {
      vi.useFakeTimers();
      const { convertElementsToGraphData } =
        await import("../../utils/graphDataUtils");
      vi.mocked(convertElementsToGraphData).mockReturnValue({
        nodes: [{ id: 1, values: [], rawValues: [], source: "query" }],
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
      const wrapper = mountWithQueryId();
      const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;
      vi.advanceTimersByTime(300);
      expect(instance?.zoomToFit).toHaveBeenCalledWith(400);

      // Fits once more (with padding) when the layout settles, only once.
      const onEngineStop = instance?.onEngineStop.mock.calls[0]?.[0];
      onEngineStop();
      onEngineStop();
      expect(instance?.zoomToFit).toHaveBeenCalledWith(400, 40);
      expect(instance?.zoomToFit).toHaveBeenCalledTimes(2);
      wrapper.unmount();
      vi.useRealTimers();
    });
  });
});
