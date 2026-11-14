import { describe, expect, it } from "vitest";
import { chainToSteps, type ChainCall, type Literal } from "./queryChain";

const str = (value: string): Literal => ({ kind: "string", value });
const int = (value: string): Literal => ({
  kind: "number",
  value,
  type: "int",
});
const call = (method: string, ...args: Literal[]): ChainCall => ({
  method,
  args,
});

describe("chainToSteps", () => {
  it("drops a final query() and accepts any query root", () => {
    expect(
      chainToSteps([
        call("insert"),
        call("nodes"),
        call("count", int("2")),
        call("query"),
      ]),
    ).toEqual({
      steps: [
        { type: "insert" },
        { type: "insert.nodes" },
        {
          type: "insert.nodes.count",
          args: [[{ selectedOption: "unsigned", value: "2" }]],
        },
      ],
    });
  });

  it.each<[string, ChainCall[], string]>([
    ["no steps", [call("query")], "The query has no steps"],
    ["an unknown root", [call("nodes")], "A query cannot start with nodes()"],
    [
      "an unknown step",
      [call("select"), call("nope")],
      "nope() cannot follow select()",
    ],
    [
      "args on a step without arguments",
      [call("select", int("1"))],
      "select() takes no arguments",
    ],
    [
      "two arguments",
      [call("select"), call("ids", int("1"), int("2"))],
      "ids() takes one argument",
    ],
    [
      "a float id",
      [
        call("select"),
        call("ids", { kind: "number", value: "1.5", type: "float" }),
      ],
      "Expected an id, got a number 1.5",
    ],
    [
      "a boolean id",
      [call("select"), call("ids", { kind: "boolean", value: true })],
      "Expected an id, got a boolean",
    ],
    [
      "a negative count",
      [call("search"), call("from", int("1")), call("limit", int("-1"))],
      "limit() takes a count of 0 or more",
    ],
    [
      "a value that is not a scalar",
      [
        call("insert"),
        call("index", { kind: "pair", key: str("a"), value: str("b") }),
      ],
      "Expected a value, got a pair",
    ],
    [
      "a nested array",
      [
        call("insert"),
        call("index", { kind: "array", items: [{ kind: "array", items: [] }] }),
      ],
      "Expected a value, got a array",
    ],
    [
      "a mixed array",
      [
        call("insert"),
        call("index", { kind: "array", items: [str("a"), int("1")] }),
      ],
      "Array values must have one type",
    ],
    [
      "commas in a string array",
      [call("insert"), call("index", { kind: "array", items: [str("a,b")] })],
      "cannot contain commas",
    ],
    [
      "a non-string regex",
      [
        call("search"),
        call("from", int("1")),
        call("where"),
        call("key", str("k")),
        call("regex", int("1")),
      ],
      "regex() takes string, not signed",
    ],
    [
      "a key-value that is not a pair",
      [
        call("insert"),
        call("values_uniform", { kind: "list", items: [str("a")] }),
      ],
      "Expected a (key, value) pair, got a string",
    ],
    [
      "several key-values for one element",
      [
        call("insert"),
        call("values", {
          kind: "list",
          items: [
            {
              kind: "list",
              items: [
                { kind: "pair", key: str("a"), value: int("1") },
                { kind: "pair", key: str("b"), value: int("2") },
              ],
            },
          ],
        }),
      ],
      "sets one key-value per element; use values_uniform()",
    ],
    [
      "an empty element",
      [
        call("insert"),
        call("values", { kind: "list", items: [{ kind: "list", items: [] }] }),
      ],
      "sets one key-value per element",
    ],
    [
      "an ordering that is not an order",
      [call("search"), call("from", int("1")), call("order_by", str("k"))],
      "Expected an ordering, got a string",
    ],
    [
      "an unknown comparison",
      [
        call("search"),
        call("from", int("1")),
        call("where"),
        call("distance", {
          kind: "comparison",
          op: "contains",
          value: int("1"),
        }),
      ],
      "distance_contains() cannot follow where()",
    ],
  ])("rejects %s", (_, chain, error) => {
    const result = chainToSteps(chain);
    expect("error" in result && result.error).toContain(error);
  });

  it("accepts a single pair as one element", () => {
    expect(
      chainToSteps([
        call("insert"),
        call("values", {
          kind: "list",
          items: [{ kind: "pair", key: str("a"), value: int("1") }],
        }),
      ]),
    ).toEqual({
      steps: [
        { type: "insert" },
        {
          type: "insert.values",
          args: [
            [
              { selectedOption: "string", value: "a" },
              { selectedOption: "signed", value: "1" },
            ],
          ],
        },
      ],
    });
  });
});
