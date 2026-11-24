import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChartBar from "./ChartBar.vue";
import type { ChartPoint } from "../../../utils/chartData";

const points: ChartPoint[] = [
  { key: "a", label: "a", value: 12 },
  { key: "b", label: "b", value: -3 },
  { key: "other", label: "Other", value: 5, other: true },
];

const mountBar = (props: Partial<InstanceType<typeof ChartBar>["$props"]>) =>
  mount(ChartBar, {
    props: {
      points,
      width: 400,
      height: 200,
      label: "Count by type",
      ...props,
    },
  });

describe("ChartBar", () => {
  it("draws columns with a value axis in wide charts with short labels", () => {
    const wrapper = mountBar({});
    expect(wrapper.find("svg").attributes("aria-label")).toBe(
      "Count by type. Use the arrow keys to read the bars.",
    );
    expect(wrapper.findAll(".bar")).toHaveLength(3);
    // Tick labels of the value axis.
    expect(wrapper.find(".grid").text()).toContain("15");
    // Values at the bar ends, the negative one below its bar.
    const values = wrapper.findAll(".value").map((v) => v.text());
    expect(values).toEqual(["12", "-3", "5"]);
    // Other is grey, the rest the series colour.
    const fills = wrapper
      .findAll(".bar path")
      .map((p) => p.attributes("style"));
    expect(fills[0]).toContain("--graph-palette-1");
    expect(fills[2]).toContain("--graph-uncolored-color");
  });

  it("keeps the values of negative bars clear of the category labels", () => {
    const signed = [
      { key: "a", label: "gain", value: 50 },
      { key: "b", label: "loss", value: -100 },
    ];
    const at = (wrapper: ReturnType<typeof mountBar>, selector: string) =>
      wrapper.findAll(selector).map((t) => {
        return {
          x: Number(t.attributes("x")),
          y: Number(t.attributes("y")),
        };
      });
    const columns = mountBar({ points: signed });
    const lowest = at(columns, ".value")[1]!;
    expect(lowest.y).toBeLessThanOrEqual(at(columns, ".category")[1]!.y - 11);

    const bars = mountBar({ points: signed, width: 200, height: 200 });
    const leftmost = at(bars, ".value")[0]!;
    const value = bars.findAll(".value")[1]!.text();
    expect(leftmost.x - value.length * 7).toBeGreaterThan(
      at(bars, ".category")[1]!.x,
    );

    const losses = mountBar({
      points: signed.map((p) => ({ ...p, value: Math.min(p.value, 0) })),
      width: 200,
      height: 200,
    });
    const zero = Number(losses.find(".baseline").attributes("x1"));
    expect(zero).toBeGreaterThan(190);
  });

  it("leaves value labels out of columns too thin for them", () => {
    const many = Array.from({ length: 12 }, (_, i) => ({
      key: `${i}`,
      label: `${i}`,
      value: 123456 * (i + 1),
    }));
    const wrapper = mountBar({ points: many, width: 300, height: 100 });
    expect(wrapper.findAll(".grid line").length).toBeGreaterThan(0);
    expect(wrapper.findAll(".value")).toHaveLength(0);
  });

  it("draws horizontal bars when labels are long or the chart is tall", () => {
    const long = [
      {
        key: "x",
        label: "a rather long category name, surely too long",
        value: 4,
      },
      { key: "y", label: "short", value: -2 },
    ];
    const wrapper = mountBar({ points: long });
    // No value axis: the values are on the bars.
    expect(wrapper.findAll(".grid line")).toHaveLength(0);
    const labels = wrapper.findAll(".category").map((l) => l.text());
    expect(labels[0]).toMatch(/…$/);
    expect(
      wrapper.findAll(".value").map((v) => v.attributes("text-anchor")),
    ).toEqual(["start", "end"]);
    // Too little room per bar: no value labels.
    const tall = mountBar({ points: long, width: 100, height: 16 });
    expect(tall.findAll(".value")).toHaveLength(0);
  });

  it("shows a tooltip on hover and the arrow keys, dimming the other bars", async () => {
    const wrapper = mountBar({});
    const bar = wrapper.find('[data-testid="chart-bar-0"]');
    expect(bar.attributes("aria-label")).toBe("a: 12");
    await bar.trigger("mouseenter");
    expect(wrapper.find('[data-testid="chart-tooltip"]').text()).toContain(
      "12",
    );
    expect(wrapper.findAll(".bar.dim")).toHaveLength(2);
    await bar.trigger("mouseleave");
    expect(wrapper.find('[data-testid="chart-tooltip"]').exists()).toBe(false);
    const horizontal = mountBar({ width: 100, height: 200 });
    // One tab stop: the bars are read with the arrow keys.
    const svg = horizontal.find("svg");
    expect(svg.attributes("tabindex")).toBe("0");
    expect(
      horizontal.find('[data-testid="chart-bar-2"]').attributes("tabindex"),
    ).toBeUndefined();
    await svg.trigger("keydown", { key: "End" });
    expect(horizontal.find('[data-testid="chart-tooltip"]').text()).toContain(
      "Other",
    );
    expect(svg.attributes("aria-activedescendant")).toBe(
      horizontal.find('[data-testid="chart-bar-2"]').attributes("id"),
    );
    await svg.trigger("keydown", { key: "ArrowLeft" });
    expect(horizontal.findAll(".bar.active")).toHaveLength(1);
    expect(horizontal.find('[data-testid="chart-bar-1"]').classes()).toContain(
      "active",
    );
    // Other keys are left to the page.
    await svg.trigger("keydown", { key: "Enter" });
    expect(horizontal.find('[data-testid="chart-bar-1"]').classes()).toContain(
      "active",
    );
    await svg.trigger("blur");
    expect(horizontal.find('[data-testid="chart-tooltip"]').exists()).toBe(
      false,
    );
  });
});
