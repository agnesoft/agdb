import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});

import {
  Comparison,
  CountComparison,
  DbKeyOrder,
  QueryBuilder,
} from "@agnesoft/agdb_api";
import type {
  QueryStep,
  QueryStepArgEntry,
  QueryType,
} from "../composables/types";
import { buildQueryFromSteps } from "./queryExecutionUtils";

const field = (selectedOption: string, value?: string) => ({
  selectedOption,
  value,
});

const step = (type: string, args?: QueryStepArgEntry[]): QueryStep => ({
  id: type,
  type: type as QueryType,
  args,
  invalid: false,
});

describe("buildQueryFromSteps", () => {
  it("throws when no steps are provided", () => {
    expect(() => buildQueryFromSteps([])).toThrow("No query steps to execute");
  });

  it("throws when a step is marked invalid", () => {
    expect(() =>
      buildQueryFromSteps([
        {
          id: "select",
          type: "select",
          invalid: true,
        },
      ]),
    ).toThrow("Cannot execute query with invalid steps");
  });

  it("builds a query with an explicit terminal step", () => {
    expect(
      buildQueryFromSteps([
        step("select"),
        step("select.ids", [[field("signed", "5")]]),
        step("query"),
      ]),
    ).toEqual(QueryBuilder.select().ids(5).query());
  });

  it("automatically terminates a query chain when needed", () => {
    expect(
      buildQueryFromSteps([
        step("select"),
        step("select.ids", [[field("signed", "5")]]),
      ]),
    ).toEqual(QueryBuilder.select().ids(5).query());
  });

  it("throws when query() is used before the chain supports it", () => {
    expect(() => buildQueryFromSteps([step("query")])).toThrow(
      "Current query chain cannot be terminated with query()",
    );
  });

  it("throws when the final chain is still not terminable", () => {
    expect(() => buildQueryFromSteps([step("insert")])).toThrow(
      "Query chain is not terminable. Add a terminal step.",
    );
  });

  it("throws for unsupported step types", () => {
    expect(() => buildQueryFromSteps([step("not-a-real-step")])).toThrow(
      "Unsupported step 'not-a-real-step' for current query chain",
    );
  });

  it("maps aliases arguments into the real builder", () => {
    expect(
      buildQueryFromSteps([
        step("insert", []),
        step("insert.aliases", [
          [field("string", "alpha")],
          [field("string", "beta")],
        ]),
        step("insert.aliases.ids", [
          [field("signed", "1")],
          [field("signed", "2")],
        ]),
      ]),
    ).toEqual(
      QueryBuilder.insert().aliases(["alpha", "beta"]).ids([1, 2]).query(),
    );
  });

  it("maps alias arguments with missing values to empty strings", () => {
    expect(
      buildQueryFromSteps([
        step("insert", []),
        step("insert.aliases", [[field("string")]]),
        step("insert.aliases.ids", [[field("signed", "1")]]),
      ]),
    ).toEqual(QueryBuilder.insert().aliases([""]).ids(1).query());
  });

  it("maps a single id argument", () => {
    expect(
      buildQueryFromSteps([
        step("select"),
        step("select.ids", [[field("signed", "5")]]),
      ]),
    ).toEqual(QueryBuilder.select().ids(5).query());
  });

  it("maps multiple id arguments including aliases", () => {
    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.ids", [
          [field("signed", "5")],
          [field("string", "alias-1")],
        ]),
      ]),
    ).toEqual(QueryBuilder.remove().ids([5, "alias-1"]).query());
  });

  it("maps count, offset, and limit arguments", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.offset", [[field("signed", "2")]]),
        step("search.limit", [[field("signed", "4")]]),
      ]),
    ).toEqual(QueryBuilder.search().elements().offset(2).limit(4).query());

    expect(
      buildQueryFromSteps([
        step("insert"),
        step("insert.nodes"),
        step("insert.nodes.count", [[field("signed", "3")]]),
      ]),
    ).toEqual(QueryBuilder.insert().nodes().count(3).query());
  });

  it("defaults missing numeric and id values to zero", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.offset", [[]]),
        step("search.limit", [[]]),
      ]),
    ).toEqual(QueryBuilder.search().elements().offset(0).limit(0).query());

    expect(
      buildQueryFromSteps([step("select"), step("select.ids", [[]])]),
    ).toEqual(QueryBuilder.select().ids(0).query());

    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.ids", [[field("signed")]]),
      ]),
    ).toEqual(QueryBuilder.remove().ids(0).query());
  });

  it("defaults missing string id values to empty strings", () => {
    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.ids", [[field("string")]]),
      ]),
    ).toEqual(QueryBuilder.remove().ids("").query());
  });

  it("maps uniform values into key-value pairs", () => {
    expect(
      buildQueryFromSteps([
        step("insert"),
        step("insert.nodes"),
        step("insert.nodes.count", [[field("signed", "1")]]),
        step("insert.nodes.values_uniform", [
          [field("string", "name"), field("string", "alice")],
          [field("string", "age"), field("unsigned", "7")],
        ]),
      ]),
    ).toEqual(
      QueryBuilder.insert()
        .nodes()
        .count(1)
        .values_uniform([
          ["name", "alice"],
          ["age", { U64: 7 }],
        ])
        .query(),
    );
  });

  it("maps select and remove values as flat lists", () => {
    expect(
      buildQueryFromSteps([
        step("select"),
        step("select.values", [[field("boolean[]", "true,false")]]),
        step("select.values.ids", [[field("signed", "1")]]),
      ]),
    ).toEqual(
      QueryBuilder.select()
        .values([["true", "false"]])
        .ids(1)
        .query(),
    );

    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.values", [[field("string[]", "a, b, ,c")]]),
        step("remove.values.ids", [[field("signed", "2")]]),
      ]),
    ).toEqual(
      QueryBuilder.remove()
        .values([["a", "b", "c"]])
        .ids(2)
        .query(),
    );
  });

  it("maps empty and custom db values through fallback coercions", () => {
    expect(
      buildQueryFromSteps([
        step("select"),
        step("select.values", [[field("string")]]),
        step("select.values.ids", [[field("signed", "1")]]),
      ]),
    ).toEqual(QueryBuilder.select().values([""]).ids(1).query());

    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.values", [[field("unsigned")]]),
        step("remove.values.ids", [[field("signed", "2")]]),
      ]),
    ).toEqual(
      QueryBuilder.remove()
        .values([{ U64: 0 }])
        .ids(2)
        .query(),
    );

    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.values", [[field("string[]")]]),
        step("remove.values.ids", [[field("signed", "3")]]),
      ]),
    ).toEqual(QueryBuilder.remove().values([[]]).ids(3).query());

    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.index", [[field("custom")]]),
        step("search.index.value", [[field("custom")]]),
      ]),
    ).toEqual(QueryBuilder.search().index("custom").value("custom").query());
  });

  it("maps nested values for insert values", () => {
    expect(
      buildQueryFromSteps([
        step("insert"),
        step("insert.values", [
          [field("string", "name"), field("string", "alice")],
          [field("string", "score"), field("float", "1.5")],
        ]),
        step("insert.values.ids", [[field("signed", "9")]]),
      ]),
    ).toEqual(
      QueryBuilder.insert()
        .values([[["name", "alice"]], [["score", { F64: 1.5 }]]])
        .ids(9)
        .query(),
    );
  });

  it("maps insert amend into nested key-value pairs", () => {
    expect(
      buildQueryFromSteps([
        step("insert"),
        step("insert.amend", [
          [field("string", "counter"), field("signed", "5")],
        ]),
        step("insert.values.ids", [[field("signed", "1")]]),
      ]),
    ).toEqual(
      QueryBuilder.insert()
        .amend([[["counter", 5]]])
        .ids(1)
        .query(),
    );
  });

  it("maps insert amend_uniform into flat key-value pairs", () => {
    expect(
      buildQueryFromSteps([
        step("insert"),
        step("insert.amend_uniform", [
          [field("string", "score"), field("unsigned", "10")],
        ]),
        step("insert.values.ids", [
          [field("signed", "1")],
          [field("signed", "2")],
        ]),
      ]),
    ).toEqual(
      QueryBuilder.insert()
        .amend_uniform([["score", { U64: 10 }]])
        .ids([1, 2])
        .query(),
    );
  });

  it("maps insert amend_or into nested key-value pairs with bitwise Or", () => {
    expect(
      buildQueryFromSteps([
        step("insert"),
        step("insert.amend_or", [
          [field("string", "flags"), field("unsigned", "4")],
        ]),
        step("insert.values.ids", [[field("signed", "1")]]),
      ]),
    ).toEqual(
      QueryBuilder.insert()
        .amend_or([[["flags", { U64: 4 }]]])
        .ids(1)
        .query(),
    );
  });

  it("maps remove amend into nested key-value pairs", () => {
    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.amend", [
          [field("string", "tags"), field("string", "old")],
        ]),
        step("remove.amend.ids", [[field("signed", "1")]]),
      ]),
    ).toEqual(
      QueryBuilder.remove()
        .amend([[["tags", "old"]]])
        .ids(1)
        .query(),
    );
  });

  it("maps remove amend_uniform with search", () => {
    expect(
      buildQueryFromSteps([
        step("remove"),
        step("remove.amend_uniform", [
          [field("string", "counter"), field("signed", "1")],
        ]),
        step("search"),
        step("search.from", [[field("string", "a")]]),
      ]),
    ).toEqual(
      QueryBuilder.remove()
        .amend_uniform([["counter", 1]])
        .search()
        .from("a")
        .query(),
    );
  });

  it("maps search from and to ids", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.from", [[field("signed", "1")]]),
        step("search.to", [[field("string", "target")]]),
      ]),
    ).toEqual(QueryBuilder.search().from(1).to("target").query());
  });

  it("maps search order_by for single and multiple entries", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.order_by", [[field("asc"), field("string", "name")]]),
      ]),
    ).toEqual(
      QueryBuilder.search().elements().order_by(DbKeyOrder.Asc("name")).query(),
    );

    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.order_by", [
          [field("asc"), field("string", "name")],
          [field("desc"), field("unsigned[]", "1,2")],
        ]),
      ]),
    ).toEqual(
      QueryBuilder.search()
        .elements()
        .order_by([DbKeyOrder.Asc("name"), DbKeyOrder.Desc({ VecU64: [1, 2] })])
        .query(),
    );
  });

  const countConditions = [
    "distance",
    "edge_count",
    "edge_count_from",
    "edge_count_to",
  ] as const;
  const countComparisons = [
    ["", CountComparison.Equal],
    ["_greater_than", CountComparison.GreaterThan],
    ["_greater_than_or_equal", CountComparison.GreaterThanOrEqual],
    ["_less_than", CountComparison.LessThan],
    ["_less_than_or_equal", CountComparison.LessThanOrEqual],
    ["_not_equal", CountComparison.NotEqual],
  ] as const;

  it.each(
    countConditions.flatMap((condition) =>
      countComparisons.map(
        ([suffix, comparison]) =>
          [`${condition}${suffix}`, condition, comparison] as const,
      ),
    ),
  )(
    "maps %s to %s with explicit CountComparison",
    (method, condition, comparison) => {
      expect(
        buildQueryFromSteps([
          step("search"),
          step("search.elements"),
          step("search.where"),
          step(`search.where.${method}`, [[field("unsigned", "3")]]),
        ]),
      ).toEqual(
        QueryBuilder.search()
          .elements()
          .where()
          [condition](comparison(3))
          .query(),
      );
    },
  );

  it("defaults missing count condition data to 0", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.where"),
        step("search.where.distance_less_than", [[]]),
      ]),
    ).toEqual(
      QueryBuilder.search()
        .elements()
        .where()
        .distance(CountComparison.LessThan(0))
        .query(),
    );
  });

  it("maps key value comparison convenience methods", () => {
    const cases = [
      [
        "value",
        Comparison.Equal({ VecF64: [1.5, 2.5] }),
        field("float[]", "1.5,2.5"),
      ],
      ["greater_than", Comparison.GreaterThan(1), field("signed", "1")],
      [
        "greater_than_or_equal",
        Comparison.GreaterThanOrEqual({ U64: 2 }),
        field("unsigned", "2"),
      ],
      ["less_than", Comparison.LessThan({ F64: 3.5 }), field("float", "3.5")],
      [
        "less_than_or_equal",
        Comparison.LessThanOrEqual(true),
        field("boolean", "true"),
      ],
      ["not_equal", Comparison.NotEqual([1, 2]), field("signed[]", "1,2")],
      ["contains", Comparison.Contains("ab"), field("string", "ab")],
      ["starts_with", Comparison.StartsWith("ab"), field("string", "ab")],
      ["ends_with", Comparison.EndsWith("ab"), field("string", "ab")],
      ["any", Comparison.Any(["a", "b"]), field("string[]", "a,b")],
      ["regex", Comparison.Regex("^a.*"), field("string", "^a.*")],
    ] as const;

    for (const [method, comparison, input] of cases) {
      expect(
        buildQueryFromSteps([
          step("search"),
          step("search.elements"),
          step("search.where"),
          step("search.where.key", [[field("string", "name")]]),
          step(`search.where.key.${method}`, [[input]]),
        ]),
      ).toEqual(
        QueryBuilder.search()
          .elements()
          .where()
          .key("name")
          .value(comparison)
          .query(),
      );
    }
  });

  it("defaults missing key value data to Equal('')", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.where"),
        step("search.where.key", [[field("string", "name")]]),
        step("search.where.key.value", [[]]),
      ]),
    ).toEqual(
      QueryBuilder.search()
        .elements()
        .where()
        .key("name")
        .value(Comparison.Equal(""))
        .query(),
    );
  });

  it("maps keys, key, index, and index.value arguments", () => {
    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.where"),
        step("search.where.keys", [
          [field("string", "a")],
          [field("unsigned", "2")],
        ]),
      ]),
    ).toEqual(
      QueryBuilder.search()
        .elements()
        .where()
        .keys(["a", { U64: 2 }])
        .query(),
    );

    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.elements"),
        step("search.where"),
        step("search.where.key", [[field("custom")]]),
        step("search.where.key.value", [[field("string", "v")]]),
      ]),
    ).toEqual(
      QueryBuilder.search()
        .elements()
        .where()
        .key("custom")
        .value(Comparison.Equal("v"))
        .query(),
    );

    expect(
      buildQueryFromSteps([
        step("search"),
        step("search.index", [[field("boolean", "false")]]),
        step("search.index.value", [[field("unsigned[]", "5,6")]]),
      ]),
    ).toEqual(
      QueryBuilder.search()
        .index(false)
        .value({ VecU64: [5, 6] })
        .query(),
    );
  });
});
