import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ColorLegend from "./ColorLegend.vue";

describe("ColorLegend", () => {
  it("groups the entries by source, with swatches and counts", async () => {
    const wrapper = mount(ColorLegend, {
      props: {
        palette: ["#ff0000", "#00ff00"],
        entries: [
          { label: "query 0", slot: 0, source: "query", count: 3 },
          { label: "role = admin", slot: 1, source: "property", count: 1 },
          { label: "node 4", slot: 0, source: "element", count: 1 },
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
});
