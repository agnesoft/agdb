import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import ColorsPanel from "./ColorsPanel.vue";
import { GRAPH_THEME_FALLBACK } from "../../utils/graphStyle";

const ramps = { ...GRAPH_THEME_FALLBACK.ramps, teal: ["#000000", "#ffffff"] };

const palette = ["#111111", "#222222", "#333333"];

const mountPanel = (props: Record<string, unknown> = {}) =>
  mount(ColorsPanel, {
    attachTo: document.body,
    props: {
      palette,
      ramps,
      queries: [
        { rowId: "r1", index: 0, label: "search.elements", slot: 1 },
        { rowId: "r2", index: 1, label: "select.ids" },
      ],
      rules: [],
      keys: [{ key: "role", values: [{ value: "admin", count: 1 }] }],
      elements: [{ id: 4, slot: 2, label: "node 4 (dave)" }],
      ...props,
    },
  });

describe("ColorsPanel", () => {
  it("colours each query, one by one or all at once", async () => {
    const wrapper = mountPanel();
    const queries = wrapper.findAll("[data-testid=colors-query]");
    expect(queries.map((q) => q.text())).toEqual([
      expect.stringContaining("search.elements"),
      expect.stringContaining("select.ids"),
    ]);
    expect(
      queries[0]
        ?.findAll("[data-testid=swatch]")[1]
        ?.attributes("aria-pressed"),
    ).toBe("true");
    await queries[1]?.findAll("[data-testid=swatch]")[0]?.trigger("click");
    await queries[0]?.get("[data-testid=swatch-none]").trigger("click");
    expect(wrapper.emitted("row-color")).toEqual([
      ["r2", 0],
      ["r1", undefined],
    ]);
    await wrapper.get("[data-testid=colors-each-query]").trigger("click");
    expect(wrapper.emitted("each-query")).toHaveLength(1);
    wrapper.unmount();
  });

  it("asks to run a query first, and has no each-query button for one query", async () => {
    const wrapper = mountPanel({ queries: [] });
    expect(wrapper.get("[data-testid=colors-queries]").text()).toContain(
      "Run a query",
    );
    await wrapper.setProps({
      queries: [{ rowId: "r1", index: 0, label: "search" }],
    });
    expect(wrapper.find("[data-testid=colors-each-query]").exists()).toBe(
      false,
    );
    wrapper.unmount();
  });

  it("passes rule changes on", async () => {
    const wrapper = mountPanel();
    await wrapper.get("[data-testid=color-rule-key]").setValue("role");
    await wrapper.get("[data-testid=color-rule-add]").trigger("click");
    expect(wrapper.emitted("update:rules")?.[0]?.[0]).toEqual([
      expect.objectContaining({ kind: "palette", key: "role" }),
    ]);
    wrapper.unmount();
  });

  it("recolours and clears single elements", async () => {
    const wrapper = mountPanel();
    const element = wrapper.get("[data-testid=colors-element]");
    expect(element.text()).toContain("node 4 (dave)");
    await element.findAll("[data-testid=swatch]")[0]?.trigger("click");
    await element.get("[data-testid=swatch-none]").trigger("click");
    expect(wrapper.emitted("element-color")).toEqual([
      [4, 0],
      [4, undefined],
    ]);
    wrapper.unmount();
  });

  it("clears everything and closes", async () => {
    const wrapper = mountPanel();
    await wrapper.get("[data-testid=color-clear-all]").trigger("click");
    expect(wrapper.emitted("clear-all")).toHaveLength(1);
    await wrapper.get("[aria-label='Close colours']").trigger("click");
    await wrapper.trigger("keydown", { key: "Escape" });
    expect(wrapper.emitted("close")).toHaveLength(2);
    wrapper.unmount();
  });

  it("colours by distance from the search start, when there is one", async () => {
    const wrapper = mountPanel();
    expect(wrapper.find("[data-testid=colors-distance-start]").exists()).toBe(
      false,
    );
    await wrapper.setProps({ hasSearchStart: true });
    await wrapper.get("[data-testid=colors-distance-start]").trigger("click");
    expect(wrapper.emitted("distance-start")).toHaveLength(1);
    wrapper.unmount();
  });
});
