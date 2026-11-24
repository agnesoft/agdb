import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
const { size } = vi.hoisted(() => ({
  size: { width: 400, height: 200 },
}));
vi.mock("@vueuse/core", async (importOriginal) => {
  const { ref } = await import("vue");
  return {
    ...(await importOriginal<typeof import("@vueuse/core")>()),
    // jsdom lays nothing out.
    useElementSize: () => ({
      width: ref(size.width),
      height: ref(size.height),
    }),
  };
});
import { mount } from "@vue/test-utils";
import ResultsChart from "./ResultsChart.vue";
import type { ChartConfig } from "../../../utils/chartConfig";
import {
  chartElement as el,
  chartResult as result,
} from "../../../utils/chartTestData";

const shop = [
  result([
    el(1, { type: "book", price: 10 }),
    el(2, { type: "book", price: 30 }),
    el(3, { type: "pen", price: 2 }),
    el(4, { price: 7 }),
  ]),
];

const config = (change: Partial<ChartConfig> = {}): ChartConfig => ({
  kind: "bar",
  groupBy: { by: "property", key: "type" },
  measure: { aggregate: "count" },
  ...change,
});

const mountChart = (
  props: Partial<InstanceType<typeof ResultsChart>["$props"]> = {},
) =>
  mount(ResultsChart, {
    props: { results: shop, ran: true, ...props },
    attachTo: document.body,
  });

const text = (wrapper: ReturnType<typeof mountChart>, testId: string) =>
  wrapper.find(`[data-testid="${testId}"]`).text();

describe("ResultsChart", () => {
  it("shows a suggested chart until one is set up", () => {
    const wrapper = mountChart();
    expect(text(wrapper, "chart-title")).toBe("Count by type");
    expect(wrapper.find('[data-testid="chart-bar"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="chart-settings"]').exists()).toBe(true);
    // The numbers for screen readers.
    expect(text(wrapper, "chart-data-table")).toContain("book2");
    expect(text(wrapper, "chart-footnote")).toBe(
      '1 element without "type" left out',
    );
  });

  it("draws each chart type", () => {
    const kinds = {
      line: "chart-line",
      donut: "chart-donut",
      stat: "chart-stat",
    } as const;
    for (const [kind, testId] of Object.entries(kinds)) {
      const wrapper = mountChart({
        config: config({ kind: kind as ChartConfig["kind"] }),
      });
      expect(wrapper.find(`[data-testid="${testId}"]`).exists()).toBe(true);
    }
    const stat = mountChart({
      config: config({
        kind: "stat",
        measure: { aggregate: "sum", key: "price" },
      }),
    });
    expect(stat.find('[data-testid="chart-stat"]').text()).toContain("49");
  });

  it("passes setting changes on", async () => {
    const wrapper = mountChart({ config: config() });
    await wrapper.find('[data-testid="chart-kind-line"]').trigger("click");
    expect(wrapper.emitted("update:config")?.[0]?.[0]).toEqual(
      config({ kind: "line" }),
    );
  });

  it("says what is missing instead of an empty chart", () => {
    const empty = (props: Parameters<typeof mountChart>[0]) =>
      mountChart(props).find('[data-testid="chart-empty"]').text();
    expect(empty({ ran: false })).toMatch(/run the query/);
    expect(empty({ results: [], config: config() })).toMatch(/no results/);
    expect(
      empty({ config: config({ groupBy: { by: "property", key: "color" } }) }),
    ).toMatch(/No element has "color"/);
    expect(
      empty({ config: config({ measure: { aggregate: "sum", key: "size" } }) }),
    ).toMatch(/number in "size"/);
    expect(
      empty({
        results: [result([], 0)],
        config: config({ kind: "donut", groupBy: { by: "query" } }),
      }),
    ).toMatch(/positive values/);
    // Values that are all zero or less are there, just not for a donut.
    expect(
      empty({
        results: [result([el(1, { type: "a", price: -3 })])],
        config: config({
          kind: "donut",
          measure: { aggregate: "sum", key: "price" },
        }),
      }),
    ).toMatch(/positive values/);
    // Grouped by query, every query has a count to show.
    const byQuery = mountChart({
      results: [result([el(1, { type: "a" })])],
      config: config({ groupBy: { by: "query" } }),
    });
    expect(byQuery.find('[data-testid="chart-empty"]').exists()).toBe(false);
  });

  it("notes what was left out of the chart", () => {
    const wrapper = mountChart({
      results: [
        result([
          ...Array.from({ length: 7 }, (_, i) =>
            el(i + 1, { type: `t${i}`, price: i - 1 }),
          ),
          el(20, { type: "x" }),
          el(21, { type: "y" }),
        ]),
      ],
      config: config({
        kind: "donut",
        measure: { aggregate: "sum", key: "price" },
      }),
    });
    expect(text(wrapper, "chart-footnote")).toBe(
      '2 elements without "type" or "price" left out · 3 smaller groups in "Other" · 1 group of zero or less not shown',
    );
  });

  it("names folded queries as queries", () => {
    const wrapper = mountChart({
      results: [1, 2, 3, 4, 5, 6].map((count) => result([], count)),
      config: config({ kind: "donut", groupBy: { by: "query" } }),
    });
    expect(text(wrapper, "chart-footnote")).toBe(
      '2 smaller queries in "Other"',
    );
  });

  it("shows a donut of an average as a sum", () => {
    const wrapper = mountChart({
      config: config({
        kind: "donut",
        measure: { aggregate: "avg", key: "price" },
      }),
    });
    expect(text(wrapper, "chart-title")).toBe("Sum of price by type");
  });

  it("changes the stored chart, not the one shown", async () => {
    const avg = config({
      kind: "donut",
      measure: { aggregate: "avg", key: "price" },
    });
    const wrapper = mountChart({ config: avg });
    await wrapper.find('[data-testid="chart-kind-bar"]').trigger("click");
    expect(wrapper.emitted("update:config")?.[0]?.[0]).toEqual({
      ...avg,
      kind: "bar",
    });
  });

  it("draws a line per query of the batch", () => {
    const batch = [
      result([el(1, { type: "book" }), el(2, { type: "pen" })]),
      result([el(3, { type: "pen" }), el(4, {})]),
    ];
    const wrapper = mountChart({
      results: batch,
      queryLabels: ["#1 books", "#2 pens"],
      config: config({ kind: "line", split: { by: "query" } }),
    });
    expect(text(wrapper, "chart-title")).toBe("Count by type, per query");
    expect(wrapper.findAll(".line")).toHaveLength(2);
    expect(wrapper.find(".line").attributes("style")).toContain(
      "--graph-palette-1",
    );
    // A column per query; "–" where a query has no such group.
    const table = wrapper.find('[data-testid="chart-data-table"]');
    expect(table.findAll("thead th").map((th) => th.text())).toEqual([
      "#1 books",
      "#2 pens",
    ]);
    expect(table.findAll("tbody tr").map((tr) => tr.text())).toEqual([
      "book1–",
      "pen11",
    ]);
    expect(text(wrapper, "chart-footnote")).toBe(
      '1 element without "type" left out',
    );
    // Missing keys say so, as for one line.
    expect(
      mountChart({
        results: batch,
        config: config({
          kind: "line",
          split: { by: "query" },
          groupBy: { by: "property", key: "colour" },
        }),
      })
        .find('[data-testid="chart-empty"]')
        .text(),
    ).toMatch(/No element has "colour"/);
  });

  it("folds the queries past the palette into a grey Other line", () => {
    const wrapper = mountChart({
      results: Array.from({ length: 6 }, (_, i) =>
        result([el(i + 1, { type: "t" })]),
      ),
      config: config({ kind: "line", split: { by: "query" } }),
    });
    const lines = wrapper.findAll(".line");
    expect(lines).toHaveLength(5);
    expect(lines[4]!.attributes("style")).toContain("--graph-uncolored-color");
    expect(text(wrapper, "chart-footnote")).toBe('2 more queries in "Other"');
  });

  it("draws a saved line by query as bars", () => {
    const wrapper = mountChart({
      config: config({ kind: "line", groupBy: { by: "query" } }),
    });
    expect(wrapper.find('[data-testid="chart-bar"]').exists()).toBe(true);
  });

  it("waits for the chart area to be laid out", () => {
    size.width = 0;
    const wrapper = mountChart();
    expect(wrapper.find('[data-testid="chart-bar"]').exists()).toBe(false);
    size.width = 400;
  });

  it("opens the settings in a popover when compact", async () => {
    const wrapper = mountChart({ compact: true, config: config() });
    const settings = () => wrapper.find('[data-testid="chart-settings"]');
    expect(settings().exists()).toBe(false);
    const toggle = wrapper.find('[data-testid="chart-settings-toggle"]');
    await toggle.trigger("click");
    expect(toggle.attributes("aria-expanded")).toBe("true");
    await wrapper.find('[data-testid="chart-kind-stat"]').trigger("click");
    expect(wrapper.emitted("update:config")).toHaveLength(1);
    await settings().trigger("keydown", { key: "Escape" });
    expect(settings().exists()).toBe(false);
    await toggle.trigger("click");
    // The outside click handler skips clicks in the same tick as the last one.
    await new Promise((resolve) => setTimeout(resolve));
    document.body.dispatchEvent(new Event("pointerdown", { bubbles: true }));
    document.body.click();
    await wrapper.vm.$nextTick();
    expect(settings().exists()).toBe(false);
    wrapper.unmount();
  });
});
