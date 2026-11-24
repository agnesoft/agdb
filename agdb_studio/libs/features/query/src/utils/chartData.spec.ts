import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import type { QueryResult } from "@agnesoft/agdb_api";
import { computeChart, computeSeries, OTHER_LABEL } from "./chartData";
import type { ChartConfig } from "./chartConfig";
import { chartElement as el, chartResult as result } from "./chartTestData";

const byType = (
  measure: ChartConfig["measure"],
  kind: ChartConfig["kind"] = "bar",
): ChartConfig => ({ kind, groupBy: { by: "property", key: "type" }, measure });

const shop = [
  result([
    el(1, { type: "book", price: 10 }),
    el(2, { type: "book", price: 30 }),
    el(3, { type: "pen", price: 2 }),
    el(4, { type: "pen", price: "n/a" }),
    el(5, { price: 99 }),
  ]),
];

const values = (results: QueryResult[], config: ChartConfig) =>
  computeChart(results, config).points.map((p) => [p.label, p.value]);

describe("chartData", () => {
  it("counts elements by a property, largest first", () => {
    const data = computeChart(shop, byType({ aggregate: "count" }));
    expect(data.points).toEqual([
      { key: "book", label: "book", value: 2 },
      { key: "pen", label: "pen", value: 2 },
    ]);
    // Element 5 has no type.
    expect(data.skipped).toBe(1);
    expect(data.folded).toBe(0);
  });

  it("folds numeric values with every aggregate", () => {
    const price = (aggregate: "sum" | "avg" | "min" | "max") =>
      values(shop, byType({ aggregate, key: "price" }));
    expect(price("sum")).toEqual([
      ["book", 40],
      ["pen", 2],
    ]);
    expect(price("avg")).toEqual([
      ["book", 20],
      ["pen", 2],
    ]);
    expect(price("min")).toEqual([
      ["book", 10],
      ["pen", 2],
    ]);
    expect(price("max")).toEqual([
      ["book", 30],
      ["pen", 2],
    ]);
    // No type (element 5) and a price that is not a number (element 4).
    expect(
      computeChart(shop, byType({ aggregate: "sum", key: "price" })).skipped,
    ).toBe(2);
  });

  it("leaves out groups without a measured number", () => {
    const results = [
      result([
        el(1, { type: "a", price: "x" }),
        el(2, { type: "b", price: 1 }),
      ]),
    ];
    expect(values(results, byType({ aggregate: "sum", key: "price" }))).toEqual(
      [["b", 1]],
    );
  });

  it("counts each element once when several queries return it", () => {
    const results = [
      result([el(1, { type: "a" })]),
      result([el(1, { type: "a" }), el(2, { type: "a" })]),
    ];
    expect(values(results, byType({ aggregate: "count" }))).toEqual([["a", 2]]);
  });

  it("charts by query, using each query's result number as its count", () => {
    const results = [
      result([], 40),
      result([el(1, { price: 3 }), el(2, { price: 4 })]),
    ];
    const config: ChartConfig = {
      kind: "bar",
      groupBy: { by: "query" },
      measure: { aggregate: "count" },
    };
    expect(computeChart(results, config, ["#1 node_count"]).points).toEqual([
      { key: "query-0", label: "#1 node_count", value: 40 },
      { key: "query-1", label: "#2", value: 2 },
    ]);
    expect(
      values(results, {
        ...config,
        measure: { aggregate: "sum", key: "price" },
      }),
    ).toEqual([["#2", 7]]);
  });

  it("measures everything as one for a stat tile", () => {
    const stat = (measure: ChartConfig["measure"]) =>
      computeChart([...shop, result([], 5)], {
        kind: "stat",
        groupBy: { by: "query" },
        measure,
      }).points;
    // The result numbers: 5 elements and a count of 5.
    expect(stat({ aggregate: "count" })).toEqual([
      { key: "all", label: "All", value: 10 },
    ]);
    expect(stat({ aggregate: "max", key: "price" })[0]?.value).toBe(99);
    expect(stat({ aggregate: "sum", key: "missing" })).toEqual([]);
  });

  it("counts an element returned by several queries once for a stat tile", () => {
    const overlapping = [
      result([el(1, { price: 1 }), el(2, { price: 2 })]),
      result([el(2, { price: 2 }), el(3, { price: 3 })]),
    ];
    const stat = (measure: ChartConfig["measure"]) =>
      computeChart(overlapping, {
        kind: "stat",
        groupBy: { by: "query" },
        measure,
      }).points[0]?.value;
    expect(stat({ aggregate: "count" })).toBe(3);
    expect(stat({ aggregate: "sum", key: "price" })).toBe(6);
  });

  it("counts a counting query by its number, as grouped by query", () => {
    // e.g. `edge_count` of two nodes: 2 elements, 10 edges.
    const batch = [
      result([el(1, { edge_count: 4 }), el(2, { edge_count: 6 })], 10),
      result([el(3, {})]),
      result([], 5),
    ];
    const count = (kind: ChartConfig["kind"]) =>
      computeChart(batch, {
        kind,
        groupBy: { by: "query" },
        measure: { aggregate: "count" },
      }).points.map((p) => p.value);
    expect(count("stat")).toEqual([16]);
    expect(count("bar")).toEqual([10, 1, 5]);
  });

  it("keeps queries in batch order unless the smallest fold into Other", () => {
    const counts = [1, 9, 3, 2, 8, 5, 4];
    const batch = counts.map((count) => result([], count));
    const byQuery = (kind: ChartConfig["kind"]) =>
      computeChart(batch, {
        kind,
        groupBy: { by: "query" },
        measure: { aggregate: "count" },
      });
    expect(byQuery("bar").points.map((p) => p.value)).toEqual(counts);
    const donut = byQuery("donut");
    expect(donut.points.map((p) => [p.label, p.value])).toEqual([
      ["#2", 9],
      ["#5", 8],
      ["#6", 5],
      ["#7", 4],
      [OTHER_LABEL, 3 + 2 + 1],
    ]);
    expect(donut.folded).toBe(3);
  });

  it("folds groups past the limit into Other", () => {
    const many = [
      result(
        Array.from({ length: 8 }, (_, i) =>
          Array.from({ length: 8 - i }, (_, j) =>
            el(i * 10 + j + 1, { type: `t${i}`, price: 1 }),
          ),
        ).flat(),
      ),
    ];
    const donut = computeChart(many, byType({ aggregate: "count" }, "donut"));
    expect(donut.points.map((p) => [p.label, p.value])).toEqual([
      ["t0", 8],
      ["t1", 7],
      ["t2", 6],
      ["t3", 5],
      [OTHER_LABEL, 4 + 3 + 2 + 1],
    ]);
    expect(donut.points.at(-1)?.other).toBe(true);
    expect(donut.folded).toBe(4);
    // Other is folded from the values, not from the groups' results.
    const avg = computeChart(
      many,
      byType({ aggregate: "avg", key: "price" }, "donut"),
    );
    expect(avg.points.at(-1)?.value).toBe(1);
    // Bars allow more groups.
    expect(computeChart(many, byType({ aggregate: "count" })).folded).toBe(0);
  });

  it("orders lines by number, then date, then natural order", () => {
    const line = (labels: (string | number)[]) => {
      const results = [
        result(labels.map((label, i) => el(i + 1, { type: label }))),
      ];
      const data = computeChart(
        results,
        byType({ aggregate: "count" }, "line"),
      );
      return [data.axis, data.points.map((p) => p.label)];
    };
    expect(line([10, 9, 100])).toEqual(["number", ["9", "10", "100"]]);
    expect(line(["2024-03-01", "2024-01-15", "2024-02-01"])).toEqual([
      "date",
      ["2024-01-15", "2024-02-01", "2024-03-01"],
    ]);
    expect(line(["item 10", "item 9", "apple"])).toEqual([
      "category",
      ["apple", "item 9", "item 10"],
    ]);
  });

  it("keeps the order of the queries along a line by query", () => {
    const data = computeChart([result([], 3), result([], 1)], {
      kind: "line",
      groupBy: { by: "query" },
      measure: { aggregate: "count" },
    });
    expect(data.axis).toBe("category");
    expect(data.points.map((p) => p.value)).toEqual([3, 1]);
  });

  describe("split by query", () => {
    const days = [
      result([
        el(1, { day: "2026-01-02", price: 5 }),
        el(2, { day: "2026-01-01", price: 3 }),
        el(3, { day: "2026-01-02", price: "n/a" }),
      ]),
      result([
        el(1, { day: "2026-01-02", price: 5 }),
        el(4, { day: "2026-01-03", price: 2 }),
        el(5, { price: 1 }),
      ]),
    ];

    it("draws a series per query along shared X, with gaps", () => {
      const data = computeSeries(days, "day", { aggregate: "count" }, ["#1 a"]);
      expect(data.axis).toBe("date");
      expect(data.xs.map((x) => x.label)).toEqual([
        "2026-01-01",
        "2026-01-02",
        "2026-01-03",
      ]);
      expect(data.series).toEqual([
        {
          key: "query-0",
          label: "#1 a",
          short: "#1",
          slot: 0,
          values: [1, 2, undefined],
        },
        // Element 1 counts in each query that returned it.
        {
          key: "query-1",
          label: "#2",
          short: "#2",
          slot: 1,
          values: [undefined, 1, 1],
        },
      ]);
      expect(data.skipped).toBe(1);
      expect(data.folded).toBe(0);
    });

    it("measures each query's groups, skipping elements without a number", () => {
      const data = computeSeries(days, "day", {
        aggregate: "sum",
        key: "price",
      });
      expect(data.series.map((s) => s.values)).toEqual([
        [3, 5, undefined],
        [undefined, 5, 2],
      ]);
      // No day on element 5, no number in element 3's price.
      expect(data.skipped).toBe(2);
    });

    it("folds the queries past the palette into Other", () => {
      const batch = Array.from({ length: 7 }, (_, i) =>
        result([el(i + 1, { type: i % 2 ? "odd" : "even", n: i })]),
      );
      const data = computeSeries(batch, "type", { aggregate: "max", key: "n" });
      expect(data.series.map((s) => [s.label, s.slot])).toEqual([
        ["#1", 0],
        ["#2", 1],
        ["#3", 2],
        ["#4", 3],
        [OTHER_LABEL, undefined],
      ]);
      // Queries 5 to 7 (n = 4, 5, 6) folded from their values.
      expect(data.xs.map((x) => x.label)).toEqual(["even", "odd"]);
      expect(data.series.at(-1)?.values).toEqual([6, 5]);
      expect(data.folded).toBe(3);
    });
  });
});
