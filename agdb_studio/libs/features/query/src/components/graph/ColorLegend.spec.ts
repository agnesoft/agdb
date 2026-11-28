import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ColorLegend from "./ColorLegend.vue";
import { GRAPH_THEME_FALLBACK } from "../../utils/graphStyle";

const ramps = { ...GRAPH_THEME_FALLBACK.ramps, teal: ["#000000", "#ffffff"] };

describe("ColorLegend", () => {
  it("groups the entries by source, with swatches and counts", async () => {
    const wrapper = mount(ColorLegend, {
      props: {
        palette: ["#ff0000", "#00ff00"],
        ramps,
        entries: [
          { label: "query 0", color: { slot: 0 }, source: "query", count: 3 },
          {
            label: "role = admin",
            color: { slot: 1 },
            source: "property",
            count: 1,
          },
          { label: "node 4", color: { slot: 0 }, source: "element", count: 1 },
        ],
      },
    });
    const groups = wrapper.findAll("[data-testid=color-legend-group]");
    expect(groups.map((g) => g.get("h6").text())).toEqual([
      "Set by hand",
      "By property",
      "By query",
    ]);
    const entries = wrapper.findAll("[data-testid=color-legend-entry]");
    expect(
      entries.map((e) => [
        e.get(".legend-label").text(),
        e.get(".legend-count").text(),
      ]),
    ).toEqual([
      ["node 4", "1"],
      ["role = admin", "1"],
      ["query 0", "3"],
    ]);
    expect(entries[1]?.get(".legend-swatch").attributes("style")).toContain(
      "rgb(0, 255, 0)",
    );

    await wrapper.get("[data-testid=color-legend-edit]").trigger("click");
    expect(wrapper.emitted("edit")).toHaveLength(1);
  });

  it("shows gradients as a scale between their ends, the strong end where it says", () => {
    const wrapper = mount(ColorLegend, {
      props: {
        palette: [],
        ramps,
        entries: [
          {
            label: "age",
            color: { ramp: 0.5, scale: "teal" },
            source: "property",
            count: 4,
            scale: { ramp: "teal", start: "10", end: "40" },
          },
          {
            label: "distance",
            color: { ramp: 1, scale: "teal" },
            source: "property",
            count: 5,
            scale: { ramp: "teal", start: "0", end: "6+", descending: true },
          },
          {
            label: "one step",
            color: { ramp: 1, scale: "teal" },
            source: "property",
            count: 1,
          },
        ],
      },
    });
    const [value, distance] = wrapper.findAll(
      "[data-testid=color-legend-scale]",
    );
    expect(value!.text()).toMatch(/10\s*40/);
    expect(value!.get(".scale-bar").attributes("style")).toContain(
      "linear-gradient(to right",
    );
    expect(distance!.text()).toMatch(/0\s*6\+/);
    expect(distance!.get(".scale-bar").attributes("style")).toContain(
      "linear-gradient(to left",
    );
    const swatch = wrapper.findAll(".legend-swatch");
    expect(swatch).toHaveLength(1);
    expect(swatch[0]!.attributes("style")).toContain("rgb(255, 255, 255)");
  });
});
