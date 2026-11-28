import { describe, expect, it } from "vitest";
import type { GraphElement, GraphNode } from "../composables/types";
import {
  copyColorConfig,
  buildColorResolver,
  emptyColorConfig,
  hasColoring,
  numericValue,
  resolveColor,
  slotColor,
  type ColorConfig,
  type ColorRef,
} from "./graphColorRules";

/** The palette slot of a colour, if it is one. */
const slotOf = (color: ColorRef | undefined) =>
  color && "slot" in color ? color.slot : undefined;

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
    expect(slotOf(resolver.colorOf(node(1)))).toBeUndefined();
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
    expect(elements.map((e) => slotOf(resolver.colorOf(e)))).toEqual([
      4,
      2,
      0,
      undefined,
    ]);
    // Only what colours something, by hand first; "role = admin" and
    // "has role" lose to the manual colour and the earlier rule.
    expect(resolver.legend).toEqual([
      { label: "node 1", color: { slot: 4 }, source: "element", count: 1 },
      { label: "has name", color: { slot: 2 }, source: "property", count: 1 },
      { label: "query second", color: { slot: 0 }, source: "query", count: 1 },
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
    expect(elements.map((e) => slotOf(resolver.colorOf(e)))).toEqual([
      0,
      1,
      0,
      2,
      undefined,
    ]);
    expect(slotOf(resolver.colorOf(node(9)))).toBeUndefined();
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
    expect(slotOf(resolver.colorOf(node(1, {}, [5, 0, 1])))).toBe(3);
    expect(slotOf(resolver.colorOf(node(2, {}, [0])))).toBeUndefined();
    expect(
      slotOf(
        resolver.colorOf({
          ...node(3),
          resultIndexes: undefined,
        } as GraphElement),
      ),
    ).toBeUndefined();
    // The colour of an element that is not shown is not in the legend.
    expect(resolver.legend).toEqual([
      { label: "r2", color: { slot: 3 }, source: "query", count: 2 },
    ]);
  });

  it("looks up slot colours", () => {
    expect(slotColor(["#111", "#222"], 1)).toBe("#222");
    expect(slotColor(["#111"], 3)).toBeUndefined();
  });
  describe("gradients", () => {
    const edge = (id: number, source: number, target: number, values = {}) =>
      ({ ...node(id, values), source, target }) as unknown as GraphElement;

    it("reads numbers, not blanks or text", () => {
      expect(numericValue(" 1.5 ")).toBe(1.5);
      expect(numericValue("-3")).toBe(-3);
      expect(numericValue("")).toBeUndefined();
      expect(numericValue("abc")).toBeUndefined();
      expect(numericValue("Infinity")).toBeUndefined();
      expect(numericValue("0x1f")).toBeUndefined();
      expect(numericValue("1e999")).toBeUndefined();
      expect(numericValue("1e3")).toBe(1000);
      expect(numericValue(".5")).toBe(0.5);
    });

    it("spreads a numeric value from its lowest to its highest shown value", () => {
      const elements = [
        node(1, { age: "20" }),
        node(2, { age: "10" }),
        node(3, { age: "40" }),
        node(4, { age: "old" }),
        node(5),
        edge(-6, 1, 2, { age: "30" }),
      ];
      const resolver = buildColorResolver(
        config({
          rules: [{ kind: "gradient", id: "g", key: "age" }],
          rows: { r: 2 },
        }),
        elements,
        ["r"],
        5,
      );
      expect(elements.map((e) => resolver.colorOf(e))).toEqual([
        { ramp: 1 / 3, scale: "teal" },
        { ramp: 0, scale: "teal" },
        { ramp: 1, scale: "teal" },
        // Without a number they fall through to the next colour.
        { slot: 2 },
        { slot: 2 },
        { ramp: 2 / 3, scale: "teal" },
      ]);
      expect(resolver.legend[0]).toEqual({
        label: "age",
        color: { ramp: 0.5, scale: "teal" },
        source: "property",
        count: 4,
        scale: { ramp: "teal", start: "10", end: "40" },
      });
    });

    it("shows a gradient's ends without float noise", () => {
      const resolver = buildColorResolver(
        config({ rules: [{ kind: "gradient", id: "g", key: "x" }] }),
        [node(1, { x: String(0.1 + 0.2) }), node(2, { x: "1.23456" })],
        [],
        5,
      );
      expect(resolver.legend[0]?.scale).toEqual({
        ramp: "teal",
        start: (0.3).toLocaleString(),
        end: (1.235).toLocaleString(),
      });
    });

    it("puts one value in the middle and skips a key without numbers", () => {
      const resolver = buildColorResolver(
        config({
          rules: [
            { kind: "gradient", id: "none", key: "name" },
            { kind: "gradient", id: "g", key: "age" },
          ],
        }),
        [node(1, { age: "5", name: "x" }), node(2, { age: "5" })],
        [],
        5,
      );
      expect(resolver.colorOf(node(1, { age: "5", name: "x" }))).toEqual({
        ramp: 0.5,
        scale: "teal",
      });
      expect(resolver.legend.map((l) => l.label)).toEqual(["age"]);
    });

    it("colours nodes by hops from the start, the start strongest and far ones together", () => {
      // A chain 1 - 2 - ... - 9, and 10 apart.
      const nodes = Array.from({ length: 10 }, (_, i) => node(i + 1));
      const links = Array.from({ length: 8 }, (_, i) =>
        edge(-(i + 1), i + 1, i + 2),
      );
      const build = (from: number | "start", startId?: number) =>
        buildColorResolver(
          config({ rules: [{ kind: "distance", id: "d", from }] }),
          [...nodes, ...links],
          [],
          5,
          { startId },
        );
      const resolver = build(1);
      expect(nodes.map((n) => resolver.colorOf(n))).toEqual([
        ...[0, 1, 2, 3, 4, 5, 6, 6, 6].map((step) => ({
          ramp: 1 - step / 6,
          scale: "teal",
        })),
        undefined,
      ]);
      // Edges are not coloured by distance.
      expect(resolver.colorOf(links[0]!)).toBeUndefined();
      // One gradient in the legend, from the start to "6+" and more.
      expect(resolver.legend).toEqual([
        {
          label: "distance",
          color: { ramp: 1, scale: "teal" },
          source: "property",
          count: 9,
          scale: { ramp: "teal", start: "0", end: "6+", descending: true },
        },
      ]);
      expect(build("start", 9).colorOf(node(9))).toEqual({
        ramp: 1,
        scale: "teal",
      });
      // A start alone is the strongest colour.
      expect(build(10).colorOf(node(10))).toEqual({ ramp: 1, scale: "teal" });
      // The gradient spans the distances shown, so near steps stay apart.
      const short = buildColorResolver(
        config({ rules: [{ kind: "distance", id: "d", from: 1 }] }),
        [...nodes.slice(0, 3), ...links.slice(0, 2)],
        [],
        5,
      );
      expect(nodes.slice(0, 3).map((n) => short.colorOf(n))).toEqual([
        { ramp: 1, scale: "teal" },
        { ramp: 0.5, scale: "teal" },
        { ramp: 0, scale: "teal" },
      ]);
      expect(short.legend.map((l) => l.scale)).toEqual([
        { ramp: "teal", start: "0", end: "2", descending: true },
      ]);
      // Exactly the last step is no "and more".
      const six = buildColorResolver(
        config({ rules: [{ kind: "distance", id: "d", from: 1 }] }),
        [...nodes.slice(0, 7), ...links.slice(0, 6)],
        [],
        5,
      );
      expect(six.legend[0]?.scale?.end).toBe("6");
      // No start, or a start that is not shown or is an edge: nothing.
      expect(build("start").legend).toEqual([]);
      expect(build(42).legend).toEqual([]);
      expect(build(-1).legend).toEqual([]);
    });

    it("resolves colours from the palette or along the gradient", () => {
      const palette = ["#111111", "#222222"];
      const ramps = {
        teal: ["#000000", "#ffffff"],
        purple: ["#000000", "#ff00ff"],
        rose: [],
        multi: [],
      };
      expect(resolveColor(palette, ramps, { slot: 1 })).toBe("#222222");
      expect(resolveColor(palette, ramps, { slot: 5 })).toBeUndefined();
      expect(resolveColor(palette, ramps, { ramp: 1, scale: "teal" })).toBe(
        "#ffffff",
      );
      expect(resolveColor(palette, ramps, { ramp: 1, scale: "purple" })).toBe(
        "#ff00ff",
      );
    });
  });

  it("copies a colour config without sharing anything", () => {
    const config = {
      rules: [{ kind: "palette" as const, id: "p", key: "role" }],
      rows: { r: 1 },
      elements: { 4: 2 },
    };
    const copy = copyColorConfig(config);
    expect(copy).toEqual(config);
    expect(copy.rules[0]).not.toBe(config.rules[0]);
    expect(copy.rows).not.toBe(config.rows);
    expect(copy.elements).not.toBe(config.elements);
  });
});
