import { describe, expect, it } from "vitest";
import { classifyResults, scalarLabel, summarizeResults } from "./resultShape";

const el = (id: number) => ({ id, from: 0, to: 0, values: [] });

describe("resultShape", () => {
  it("summarizes nodes, edges and other elements per result", () => {
    expect(
      summarizeResults([
        { result: 3, elements: [el(1), el(-2), el(0)] },
        { result: 0, elements: [] },
      ]),
    ).toEqual([
      { index: 0, result: 3, nodes: 1, edges: 1, other: 1 },
      { index: 1, result: 0, nodes: 0, edges: 0, other: 0 },
    ]);
  });

  it("uses the scalar view for results without elements", () => {
    expect(classifyResults([{ result: 10, elements: [] }], ["select"])).toBe(
      "scalar",
    );
    expect(classifyResults([], [])).toBe("scalar");
  });

  it("uses the table view for metadata queries", () => {
    expect(
      classifyResults(
        [{ result: 1, elements: [el(1)] }],
        ["select", "select.keys"],
      ),
    ).toBe("table");
  });

  it("uses the table view when only id 0 elements are returned", () => {
    expect(
      classifyResults([{ result: 1, elements: [el(0)] }], ["select"]),
    ).toBe("table");
  });

  it("uses the graph view for graph elements", () => {
    expect(
      classifyResults([{ result: 2, elements: [el(1), el(-2)] }], ["select"]),
    ).toBe("graph");
  });

  it("labels scalar results", () => {
    expect(scalarLabel(["select", "select.node_count"], false)).toBe(
      "Node count",
    );
    expect(scalarLabel(["insert"], true)).toBe("Affected");
    expect(scalarLabel(["select"], false)).toBe("Result");
  });
});
