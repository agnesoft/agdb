import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChartDonut from "./ChartDonut.vue";
import type { ChartPoint } from "../../../utils/chartData";

const points: ChartPoint[] = [
  { key: "a", label: "a", value: 3 },
  { key: "b", label: "b", value: 1 },
  { key: "other", label: "Other", value: 4, other: true },
];

const mountDonut = (
  props: Partial<InstanceType<typeof ChartDonut>["$props"]>,
) =>
  mount(ChartDonut, {
    props: {
      points,
      width: 400,
      height: 200,
      label: "Count by type",
      ...props,
    },
  });

describe("ChartDonut", () => {
  it("draws slices with the total and a legend", () => {
    const wrapper = mountDonut({});
    expect(wrapper.findAll(".slice")).toHaveLength(3);
    expect(wrapper.find(".total").text()).toBe("8");
    expect(wrapper.findAll(".legend li").map((li) => li.text())).toEqual([
      "a3",
      "b1",
      "Other4",
    ]);
    expect(wrapper.classes()).toContain("side");
    // Palette colours in order, Other grey.
    const fills = wrapper.findAll(".slice").map((s) => s.attributes("style"));
    expect(fills[1]).toContain("--graph-palette-2");
    expect(fills[2]).toContain("--graph-uncolored-color");
    // The slice over half the ring takes the large arc.
    expect(wrapper.findAll(".slice")[2]!.attributes("aria-label")).toBe(
      "Other: 4, 50%",
    );
  });

  it("puts the legend below the ring in tall charts", () => {
    expect(mountDonut({ width: 200, height: 400 }).classes()).not.toContain(
      "side",
    );
  });

  it("draws a single slice as a full ring", () => {
    const wrapper = mountDonut({
      points: [{ key: "a", label: "a", value: 2 }],
    });
    // Two half arcs outside, two inside.
    expect(wrapper.find(".slice").attributes("d")?.match(/A/g)).toHaveLength(4);
  });

  it("shows a slice's value and share on hover, the arrow keys and legend hover", async () => {
    const wrapper = mountDonut({});
    const tooltip = () => wrapper.find('[data-testid="chart-tooltip"]');
    const slice = wrapper.find('[data-testid="chart-slice-0"]');
    await slice.trigger("mouseenter");
    expect(tooltip().text()).toContain("37.5%");
    expect(wrapper.findAll(".slice.dim")).toHaveLength(2);
    await slice.trigger("mouseleave");
    expect(tooltip().exists()).toBe(false);
    const svg = wrapper.find("svg");
    await svg.trigger("keydown", { key: "ArrowRight" });
    expect(tooltip().text()).toContain("37.5%");
    expect(svg.attributes("aria-activedescendant")).toBe(
      slice.attributes("id"),
    );
    await svg.trigger("keydown", { key: "Tab" });
    expect(slice.classes()).toContain("active");
    await svg.trigger("blur");
    expect(tooltip().exists()).toBe(false);
    const legend = wrapper.findAll(".legend li")[1]!;
    await legend.trigger("mouseenter");
    expect(tooltip().text()).toContain("b");
    expect(wrapper.findAll(".legend li.dim")).toHaveLength(2);
    await legend.trigger("mouseleave");
    expect(tooltip().exists()).toBe(false);
  });
});
