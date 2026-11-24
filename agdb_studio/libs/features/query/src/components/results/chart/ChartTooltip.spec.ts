import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ChartTooltip from "./ChartTooltip.vue";

const tooltipAt = (x: number, detail?: string) =>
  mount(ChartTooltip, {
    props: { x, y: 50, width: 400, label: "type a", value: "12", detail },
  });

describe("ChartTooltip", () => {
  it("shows the label, value and detail above the mark", () => {
    const wrapper = tooltipAt(200, "40 %");
    expect(wrapper.text()).toBe("type a1240 %");
    expect(wrapper.attributes("style")).toContain("left: 200px");
    expect(wrapper.attributes("style")).toContain("translate(-50%");
    expect(tooltipAt(200).find(".detail").exists()).toBe(false);
  });

  it("opens away from the chart's edges", () => {
    expect(tooltipAt(10).attributes("style")).toContain("translate(-0.75rem");
    expect(tooltipAt(390).attributes("style")).toContain(
      "translate(calc(-100% + 0.75rem)",
    );
  });

  it("opens below the mark where there is no room above", () => {
    const at = (y: number, rows = 0) =>
      mount(ChartTooltip, {
        props: {
          x: 200,
          y,
          width: 400,
          label: "2026-05",
          rows: Array.from({ length: rows }, (_, i) => ({
            key: `${i}`,
            label: `#${i}`,
            value: "1",
            color: "red",
          })),
        },
      }).attributes("style");
    expect(at(10)).toContain("translate(-50%, 0.75rem)");
    expect(at(50)).toContain("calc(-100% - 0.5rem)");
    // Each row makes it taller.
    expect(at(50, 3)).toContain("translate(-50%, 0.75rem)");
  });
});
