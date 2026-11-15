import { describe, expect, it } from "vitest";
import type { GraphElement, GraphNode } from "../composables/types";
import {
  buildColorResolver,
  emptyColorConfig,
  hasColoring,
  slotColor,
  type ColorConfig,
} from "./graphColorRules";

const node = (
  id: number,
  values: Record<string, string> = {},
  resultIndexes: number[] = [0],
): GraphElement =>
  ({
    id,
    elementSource: "query",
    values: Object.entries(values).map(([key, value]) => ({ key, value })),
    resultIndexes,
  }) as unknown as GraphNode;

const config = (patch: Partial<ColorConfig>): ColorConfig => ({
  ...emptyColorConfig(),
  ...patch,
});

describe("graphColorRules", () => {
  it("reports whether a config colours anything", () => {
    expect(hasColoring(undefined)).toBe(false);
    expect(hasColoring(emptyColorConfig())).toBe(false);
    expect(hasColoring(config({ rows: { r: 0 } }))).toBe(true);
    expect(hasColoring(config({ elements: { 1: 0 } }))).toBe(true);
    expect(
      hasColoring(config({ rules: [{ kind: "palette", id: "p", key: "k" }] })),
    ).toBe(true);
  });

  it("colours nothing without a config", () => {
    const resolver = buildColorResolver(undefined, [node(1)], [], 5);
    expect(resolver.slotOf(node(1))).toBeUndefined();
    expect(resolver.legend).toEqual([]);
  });

  it("applies manual colours, then rules in order, then rows", () => {
    const elements = [
      node(1, { role: "admin" }),
      node(2, { role: "user", name: "b" }),
      node(3, {}, [1]),
      node(4, {}, [0]),
    ];
    const resolver = buildColorResolver(
      config({
        elements: { 1: 4 },
        rules: [
          {
            kind: "property",
            id: "a",
            key: "role",
            op: "eq",
            value: "admin",
            color: 1,
          },
          { kind: "property", id: "b", key: "name", op: "exists", color: 2 },
          { kind: "property", id: "c", key: "role", op: "exists", color: 3 },
        ],
        rows: { second: 0 },
      }),
      elements,
      ["first", "second"],
      5,
      { row: (rowId) => `query ${rowId}`, element: (id) => `node ${id}` },
    );
    expect(elements.map((e) => resolver.slotOf(e))).toEqual([
      4,
      2,
      0,
      undefined,
    ]);
    // Only what colours something, by hand first; "role = admin" and
    // "has role" lose to the manual colour and the earlier rule.
    expect(resolver.legend).toEqual([
      { label: "node 1", slot: 4, source: "element", count: 1 },
      { label: "has name", slot: 2, source: "property", count: 1 },
      { label: "query second", slot: 0, source: "query", count: 1 },
    ]);
  });

  it("gives distinct values palette slots by first appearance, none beyond the palette", () => {
    const elements = ["a", "b", "a", "c", "d"].map((role, i) =>
      node(i, { role }),
    );
    const resolver = buildColorResolver(
      config({ rules: [{ kind: "palette", id: "p", key: "role" }] }),
      [...elements, node(9)],
      [],
      3,
    );
    expect(elements.map((e) => resolver.slotOf(e))).toEqual([
      0,
      1,
      0,
      2,
      undefined,
    ]);
    expect(resolver.slotOf(node(9))).toBeUndefined();
    expect(resolver.legend.map((l) => [l.label, l.count])).toEqual([
      ["role = a", 2],
      ["role = b", 1],
      ["role = c", 1],
    ]);
  });

  it("uses the first coloured row of an element and falls back when a row is unknown", () => {
    const shown = [node(1, {}, [5, 0, 1]), node(5, {}, [1])];
    const resolver = buildColorResolver(
      config({ rows: { r2: 3 }, elements: { 9: 1 } }),
      shown,
      ["r1", "r2"],
      5,
    );
    expect(resolver.slotOf(node(1, {}, [5, 0, 1]))).toBe(3);
    expect(resolver.slotOf(node(2, {}, [0]))).toBeUndefined();
    expect(
      resolver.slotOf({ ...node(3), resultIndexes: undefined } as GraphElement),
    ).toBeUndefined();
    // The colour of an element that is not shown is not in the legend.
    expect(resolver.legend).toEqual([
      { label: "r2", slot: 3, source: "query", count: 2 },
    ]);
  });

  it("looks up slot colours", () => {
    expect(slotColor(["#111", "#222"], 1)).toBe("#222");
    expect(slotColor(["#111"], 3)).toBeUndefined();
  });
});
