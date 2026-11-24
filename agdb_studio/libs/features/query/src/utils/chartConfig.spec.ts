import { describe, expect, it, vi } from "vitest";
import type { DbValue } from "@agnesoft/agdb_api";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import {
  chartLabel,
  copyChartConfig,
  dbValueNumber,
  isSplit,
  measureLabel,
  numericValue,
  readChartConfig,
  resultKeys,
  showableChart,
  suggestChart,
  type ChartConfig,
} from "./chartConfig";
import { chartElement as el, chartResult as result } from "./chartTestData";

describe("chartConfig", () => {
  it("labels measures and charts", () => {
    const bar: ChartConfig = {
      kind: "bar",
      groupBy: { by: "property", key: "type" },
      measure: { aggregate: "avg", key: "price" },
    };
    expect(measureLabel({ aggregate: "count" })).toBe("Count");
    expect(chartLabel(bar)).toBe("Average of price by type");
    expect(
      chartLabel({
        ...bar,
        groupBy: { by: "query" },
        measure: { aggregate: "count" },
      }),
    ).toBe("Count by query");
    expect(
      chartLabel({
        ...bar,
        kind: "stat",
        measure: { aggregate: "sum", key: "price" },
      }),
    ).toBe("Sum of price");
  });

  it("reads numbers from numbers and numeric strings only", () => {
    expect(numericValue(3)).toBe(3);
    expect(numericValue(Infinity)).toBeUndefined();
    expect(numericValue(" 2.5 ")).toBe(2.5);
    expect(numericValue("")).toBeUndefined();
    expect(numericValue("abc")).toBeUndefined();
    expect(numericValue(true)).toBeUndefined();
  });

  it.each<[DbValue, number | undefined]>([
    [{ I64: -2 }, -2],
    [{ U64: 3 }, 3],
    [{ F64: 1.5 }, 1.5],
    [{ String: "4" }, 4],
    [{ String: "true" }, undefined],
    [{ VecI64: [1] }, undefined],
    ["None", undefined],
  ])("reads the number of %j", (value, expected) => {
    expect(dbValueNumber(value)).toBe(expected);
  });

  it("lists the keys of the results once per element", () => {
    const results = [
      result([
        el(1, { type: "a", price: 5 }),
        el(2, { type: "b", price: "x" }),
      ]),
      result([el(1, { type: "a", price: 5 }), el(3, { size: "7" })]),
    ];
    expect(resultKeys(results)).toEqual([
      { key: "type", count: 2, numeric: false, textual: true, distinct: 2 },
      // Some prices are numbers: measurable, the rest left out.
      { key: "price", count: 2, numeric: true, textual: true, distinct: 2 },
      { key: "size", count: 1, numeric: true, textual: false, distinct: 1 },
    ]);
  });

  it("suggests a chart that fits the results", () => {
    const count = { aggregate: "count" };
    // A batch of counts: one bar per query.
    expect(suggestChart([result([], 4), result([], 7)])).toEqual({
      kind: "bar",
      groupBy: { by: "query" },
      measure: count,
    });
    // One count: a stat tile.
    expect(suggestChart([result([], 4)]).kind).toBe("stat");
    // Elements: counted by the first property with a few shared values;
    // a name unique to each element would chart a bar of 1 each.
    const typed = [
      el(1, { id: 1, name: "x", type: "a" }),
      el(2, { id: 2, name: "y", type: "a" }),
      el(3, { id: 3, name: "z", type: "b" }),
    ];
    expect(suggestChart([result(typed)])).toEqual({
      kind: "bar",
      groupBy: { by: "property", key: "type" },
      measure: count,
    });
    // Nothing to group by: per query for batches, else a stat tile.
    const plain = [el(1, { n: 1 }), el(2, { n: 2 })];
    expect(suggestChart([result(plain)]).kind).toBe("stat");
    expect(suggestChart([result(plain), result(plain)])).toMatchObject({
      kind: "bar",
      groupBy: { by: "query" },
    });
  });

  it("reads stored configs, rejecting malformed ones", () => {
    const line = {
      kind: "line",
      groupBy: { by: "property", key: "day" },
      measure: { aggregate: "max", key: "load" },
    };
    expect(readChartConfig(line)).toEqual(line);
    // Extra fields are dropped.
    expect(
      readChartConfig({
        kind: "bar",
        groupBy: { by: "query", key: "x" },
        measure: { aggregate: "count", key: "y" },
      }),
    ).toEqual({
      kind: "bar",
      groupBy: { by: "query" },
      measure: { aggregate: "count" },
    });
    expect(readChartConfig(null)).toBeUndefined();
    expect(readChartConfig({ ...line, measure: 1 })).toBeUndefined();
    expect(readChartConfig({ ...line, kind: "pie" })).toBeUndefined();
    expect(
      readChartConfig({ ...line, groupBy: { by: "property" } }),
    ).toBeUndefined();
    expect(
      readChartConfig({ ...line, measure: { aggregate: "max", key: "" } }),
    ).toBeUndefined();
  });

  it("shows a donut of measures that add up only", () => {
    const donut = (measure: ChartConfig["measure"]): ChartConfig => ({
      kind: "donut",
      groupBy: { by: "property", key: "type" },
      measure,
    });
    const count = donut({ aggregate: "count" });
    expect(showableChart(count)).toBe(count);
    const sum = donut({ aggregate: "sum", key: "price" });
    expect(showableChart(sum)).toBe(sum);
    expect(showableChart(donut({ aggregate: "avg", key: "price" }))).toEqual(
      sum,
    );
    const bar: ChartConfig = {
      ...sum,
      kind: "bar",
      measure: { aggregate: "max", key: "price" },
    };
    expect(showableChart(bar)).toBe(bar);
  });

  it("splits lines by property per query only", () => {
    const line: ChartConfig = {
      kind: "line",
      groupBy: { by: "property", key: "day" },
      measure: { aggregate: "count" },
      split: { by: "query" },
    };
    expect(isSplit(line)).toBe(true);
    expect(chartLabel(line)).toBe("Count by day, per query");
    // Kept, but ignored, by other kinds.
    expect(isSplit({ ...line, kind: "bar" })).toBe(false);
    expect(chartLabel({ ...line, kind: "bar" })).toBe("Count by day");
    expect(isSplit({ ...line, split: undefined })).toBe(false);
    // Read back, and an unknown split left out.
    expect(readChartConfig(line)).toEqual(line);
    expect(readChartConfig({ ...line, split: { by: "day" } })).toEqual({
      kind: "line",
      groupBy: line.groupBy,
      measure: line.measure,
    });
    const copy = copyChartConfig(line);
    expect(copy).toEqual(line);
    expect(copy.split).not.toBe(line.split);
  });

  it("shows a line by query as bars", () => {
    const line: ChartConfig = {
      kind: "line",
      groupBy: { by: "query" },
      measure: { aggregate: "count" },
    };
    expect(showableChart(line)).toEqual({ ...line, kind: "bar" });
  });

  it("copies configs", () => {
    const config: ChartConfig = {
      kind: "donut",
      groupBy: { by: "query" },
      measure: { aggregate: "count" },
    };
    const copy = copyChartConfig(config);
    expect(copy).toEqual(config);
    expect(copy.groupBy).not.toBe(config.groupBy);
  });
});
