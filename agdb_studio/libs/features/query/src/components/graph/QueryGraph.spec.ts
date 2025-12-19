import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QueryGraph from "./QueryGraph.vue";
import ForceGraph3D from "3d-force-graph";
import { nextTick } from "vue";
vi.mock("3d-force-graph", () => {
  const instance = {
    graphData: vi.fn().mockReturnThis(),
    nodeAutoColorBy: vi.fn().mockReturnThis(),
    linkAutoColorBy: vi.fn().mockReturnThis(),
    nodeLabel: vi.fn().mockReturnThis(),
    linkLabel: vi.fn().mockReturnThis(),
    height: vi.fn().mockReturnThis(),
    width: vi.fn().mockReturnThis(),
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
  });
  it("renders the query graph", () => {
    const wrapper = mount(QueryGraph);
    expect(wrapper.find(".query-graph").exists()).toBe(true);
  });

  it("initializes 3d-force-graph on mount", async () => {
    const wrapper = mount(QueryGraph, { attachTo: document.body });
    await nextTick();

    expect(ForceGraph3D).toHaveBeenCalledTimes(1);
    const instance = vi.mocked(ForceGraph3D).mock.results[0]?.value;

    expect(instance?.graphData).toHaveBeenCalled();
    expect(instance?.nodeAutoColorBy).toHaveBeenCalledWith("id");
    expect(instance?.linkAutoColorBy).toHaveBeenCalledWith("source");
    expect(instance?.nodeLabel).toHaveBeenCalled();
    expect(instance?.linkLabel).toHaveBeenCalled();
    expect(instance?.height).toHaveBeenCalled();
    expect(instance?.width).toHaveBeenCalled();

    wrapper.unmount();
  });
});
