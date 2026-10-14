import { describe, expect, it, vi } from "vitest";
import type { Components } from "@agnesoft/agdb_api";

vi.mock(import("@agnesoft/agdb_api"), async (importOriginal) => {
  const actual = await importOriginal();
  return {
    ...actual,
  };
});

import {
  convertElementsToGraphData,
  dbValueToString,
  mergeGraphData,
} from "./graphDataUtils";

const strVal = (s: string): Components.Schemas.DbValue => ({ String: s });
const i64Val = (n: number): Components.Schemas.DbValue => ({ I64: n });
const u64Val = (n: number): Components.Schemas.DbValue => ({ U64: n });
const f64Val = (n: number): Components.Schemas.DbValue => ({ F64: n });
const vecStrVal = (arr: string[]): Components.Schemas.DbValue => ({
  VecString: arr,
});
const vecI64Val = (arr: number[]): Components.Schemas.DbValue => ({
  VecI64: arr,
});
const bytesVal = (arr: number[]): Components.Schemas.DbValue => ({
  Bytes: arr,
});

describe("dbValueToString", () => {
  it("converts a String value", () => {
    expect(dbValueToString(strVal("hello"))).toBe("hello");
  });

  it("converts an I64 value", () => {
    expect(dbValueToString(i64Val(42))).toBe("42");
  });

  it("converts a U64 value", () => {
    expect(dbValueToString(u64Val(100))).toBe("100");
  });

  it("converts an F64 value", () => {
    expect(dbValueToString(f64Val(3.14))).toBe("3.14");
  });

  it("converts a VecString value", () => {
    expect(dbValueToString(vecStrVal(["a", "b", "c"]))).toBe("a, b, c");
  });

  it("converts a VecI64 value", () => {
    expect(dbValueToString(vecI64Val([1, 2, 3]))).toBe("1, 2, 3");
  });

  it("converts a Bytes value", () => {
    expect(dbValueToString(bytesVal([1, 2, 3]))).toBe("1, 2, 3");
  });

  it("converts a boolean-like string", () => {
    expect(dbValueToString(strVal("true"))).toBe("true");
    expect(dbValueToString(strVal("false"))).toBe("false");
  });
});

describe("convertElementsToGraphData", () => {
  it("converts positive-id elements to nodes", () => {
    const elements: Components.Schemas.DbElement[] = [
      {
        id: 1,
        from: 0,
        to: 0,
        values: [{ key: strVal("name"), value: strVal("Alice") }],
      },
      {
        id: 2,
        from: 0,
        to: 0,
        values: [{ key: strVal("name"), value: strVal("Bob") }],
      },
    ];

    const result = convertElementsToGraphData(elements, "query");

    expect(result.nodes).toHaveLength(2);
    expect(result.links).toHaveLength(0);
    expect(result.nodes[0]).toEqual({
      id: 1,
      values: [{ key: "name", value: "Alice" }],
      source: "query",
    });
    expect(result.nodes[1]).toEqual({
      id: 2,
      values: [{ key: "name", value: "Bob" }],
      source: "query",
    });
  });

  it("converts negative-id elements to links", () => {
    const elements: Components.Schemas.DbElement[] = [
      {
        id: -10,
        from: 1,
        to: 2,
        values: [{ key: strVal("role"), value: strVal("owns") }],
      },
    ];

    const result = convertElementsToGraphData(elements, "context");

    expect(result.nodes).toHaveLength(0);
    expect(result.links).toHaveLength(1);
    expect(result.links[0]).toEqual({
      id: -10,
      values: [{ key: "role", value: "owns" }],
      source: 1,
      target: 2,
      elementSource: "context",
    });
  });

  it("skips elements with id === 0", () => {
    const elements: Components.Schemas.DbElement[] = [
      { id: 0, from: 0, to: 0, values: [] },
    ];

    const result = convertElementsToGraphData(elements, "query");

    expect(result.nodes).toHaveLength(0);
    expect(result.links).toHaveLength(0);
  });

  it("handles mixed nodes and edges", () => {
    const elements: Components.Schemas.DbElement[] = [
      { id: 5, from: 0, to: 0, values: [] },
      { id: -1, from: 5, to: 10, values: [] },
      { id: 10, from: 0, to: 0, values: [] },
    ];

    const result = convertElementsToGraphData(elements, "query");

    expect(result.nodes).toHaveLength(2);
    expect(result.links).toHaveLength(1);
    expect(result.nodes.map((n) => n.id)).toEqual([5, 10]);
    expect(result.links[0]?.source).toBe(5);
    expect(result.links[0]?.target).toBe(10);
  });

  it("returns empty data for empty input", () => {
    const result = convertElementsToGraphData([], "query");

    expect(result).toEqual({ nodes: [], links: [] });
  });
});

describe("mergeGraphData", () => {
  it("merges disjoint sets", () => {
    const query = {
      nodes: [{ id: 1, values: [], source: "query" as const }],
      links: [
        {
          id: -1,
          values: [],
          source: 1,
          target: 2,
          elementSource: "query" as const,
        },
      ],
    };
    const context = {
      nodes: [{ id: 2, values: [], source: "context" as const }],
      links: [
        {
          id: -2,
          values: [],
          source: 2,
          target: 3,
          elementSource: "context" as const,
        },
      ],
    };

    const result = mergeGraphData(query, context);

    expect(result.nodes).toHaveLength(2);
    expect(result.links).toHaveLength(2);
  });

  it("gives query elements priority over context on overlap", () => {
    const query = {
      nodes: [
        {
          id: 1,
          values: [{ key: "name", value: "from-query" }],
          source: "query" as const,
        },
      ],
      links: [],
    };
    const context = {
      nodes: [
        {
          id: 1,
          values: [{ key: "name", value: "from-context" }],
          source: "context" as const,
        },
      ],
      links: [],
    };

    const result = mergeGraphData(query, context);

    expect(result.nodes).toHaveLength(1);
    expect(result.nodes[0]?.source).toBe("query");
    expect(result.nodes[0]?.values[0]?.value).toBe("from-query");
  });

  it("gives query links priority over context on overlap", () => {
    const query = {
      nodes: [],
      links: [
        {
          id: -5,
          values: [{ key: "type", value: "query-edge" }],
          source: 1,
          target: 2,
          elementSource: "query" as const,
        },
      ],
    };
    const context = {
      nodes: [],
      links: [
        {
          id: -5,
          values: [{ key: "type", value: "context-edge" }],
          source: 1,
          target: 2,
          elementSource: "context" as const,
        },
      ],
    };

    const result = mergeGraphData(query, context);

    expect(result.links).toHaveLength(1);
    expect(result.links[0]?.elementSource).toBe("query");
  });

  it("returns empty data when both inputs are empty", () => {
    const result = mergeGraphData(
      { nodes: [], links: [] },
      { nodes: [], links: [] },
    );

    expect(result).toEqual({ nodes: [], links: [] });
  });
});
