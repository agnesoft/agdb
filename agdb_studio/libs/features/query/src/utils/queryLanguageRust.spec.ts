import { describe, expect, it } from "vitest";
import { rustLanguage } from "./queryLanguageRust";
import { chainToSteps, stepsToChain } from "./queryChain";
import type { CopiedStep } from "../composables/types";

const f = (selectedOption: string, value?: string) =>
  value === undefined ? { selectedOption } : { selectedOption, value };

const toRust = (steps: CopiedStep[]) =>
  rustLanguage.format(stepsToChain(steps));

const fromRust = (text: string) => {
  const parsed = rustLanguage.parse(text);
  return "calls" in parsed ? chainToSteps(parsed.calls) : parsed;
};

describe("rustLanguage", () => {
  it.each<[string, CopiedStep[]]>([
    [
      "QueryBuilder::select().node_count().query()",
      [{ type: "select" }, { type: "select.node_count" }],
    ],
    [
      'QueryBuilder::select().ids([QueryId::from(1), QueryId::from(-2), QueryId::from("alias")]).query()',
      [
        { type: "select" },
        {
          type: "select.ids",
          args: [
            [f("signed", "1")],
            [f("signed", "-2")],
            [f("string", "alias")],
          ],
        },
      ],
    ],
    [
      'QueryBuilder::select().values(["key", "key2"]).ids(1).query()',
      [
        { type: "select" },
        {
          type: "select.values",
          args: [[f("string", "key")], [f("string", "key2")]],
        },
        { type: "select.values.ids", args: [[f("signed", "1")]] },
      ],
    ],
    [
      'QueryBuilder::insert().nodes().aliases(["a", "b"]).values_uniform([("k", 1_u64).into(), ("f", 1.5).into(), ("b", true).into()]).query()',
      [
        { type: "insert" },
        { type: "insert.nodes" },
        {
          type: "insert.nodes.aliases",
          args: [[f("string", "a")], [f("string", "b")]],
        },
        {
          type: "insert.nodes.values_uniform",
          args: [
            [f("string", "k"), f("unsigned", "1")],
            [f("string", "f"), f("float", "1.5")],
            [f("string", "b"), f("boolean", "true")],
          ],
        },
      ],
    ],
    [
      'QueryBuilder::insert().values([[("k", vec![1, 2]).into()], [("s", vec!["x", "y"]).into()]]).ids(1).query()',
      [
        { type: "insert" },
        {
          type: "insert.values",
          args: [
            [f("string", "k"), f("signed[]", "1, 2")],
            [f("string", "s"), f("string[]", "x, y")],
          ],
        },
        { type: "insert.values.ids", args: [[f("signed", "1")]] },
      ],
    ],
    [
      'QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into()), DbKeyOrder::Desc(2.into())]).limit(10).where_().key("k").greater_than(2.5).query()',
      [
        { type: "search" },
        { type: "search.from", args: [[f("signed", "1")]] },
        {
          type: "search.order_by",
          args: [
            [f("asc"), f("string", "k")],
            [f("desc"), f("signed", "2")],
          ],
        },
        { type: "search.limit", args: [[f("unsigned", "10")]] },
        { type: "search.where" },
        { type: "search.where.key", args: [[f("string", "k")]] },
        { type: "search.where.key.greater_than", args: [[f("float", "2.5")]] },
      ],
    ],
    [
      'QueryBuilder::search().from("root").where_().distance_less_than(3).and().not().edge().query()',
      [
        { type: "search" },
        { type: "search.from", args: [[f("string", "root")]] },
        { type: "search.where" },
        {
          type: "search.where.distance_less_than",
          args: [[f("unsigned", "3")]],
        },
        { type: "search.where.and" },
        { type: "search.where.not" },
        { type: "search.where.edge" },
      ],
    ],
    [
      // Steps whose arguments are not filled in yet.
      "QueryBuilder::select().ids().query()",
      [{ type: "select" }, { type: "select.ids" }],
    ],
  ])("formats and parses %s", (code, steps) => {
    expect(toRust(steps)).toBe(code);
    expect(fromRust(code)).toEqual({ steps });
  });

  it("formats mixed lists, escapes strings and keeps an explicit query()", () => {
    expect(
      toRust([
        { type: "select" },
        {
          type: "select.values",
          args: [[f("string", 'a"\\\n\t\r\u0001')], [f("unsigned", "1")]],
        },
        { type: "select.values.ids", args: [[f("signed", "1")]] },
        { type: "query" },
      ]),
    ).toBe(
      'QueryBuilder::select().values([DbValue::from("a\\"\\\\\\n\\t\\r\\u{01}"), DbValue::from(1_u64)]).ids(1).query()',
    );
  });

  it("formats floats and empty values", () => {
    expect(
      toRust([
        { type: "search" },
        { type: "search.index", args: [[f("float", "2")]] },
        { type: "search.index.value", args: [[f("float[]", "1, 2.5e3")]] },
      ]),
    ).toBe("QueryBuilder::search().index(2.0).value(vec![1.0, 2.5e3]).query()");
    expect(
      toRust([
        { type: "search" },
        { type: "search.index", args: [[f("signed")]] },
        { type: "search.index.value", args: [[f("boolean[]", "true, x")]] },
      ]),
    ).toBe("QueryBuilder::search().index(0).value(vec![true, false]).query()");
    expect(
      toRust([
        { type: "insert" },
        { type: "insert.nodes" },
        { type: "insert.nodes.count", args: [[f("unsigned")]] },
      ]),
    ).toBe("QueryBuilder::insert().nodes().count(0).query()");
    expect(
      toRust([
        { type: "select" },
        { type: "select.ids", args: [[f("string")]] },
      ]),
    ).toBe('QueryBuilder::select().ids("").query()');
    expect(
      toRust([
        { type: "select" },
        { type: "select.ids", args: [[{ selectedOption: "signed" }]] },
      ]),
    ).toBe("QueryBuilder::select().ids(0).query()");
  });

  it("formats steps with missing fields", () => {
    expect(
      toRust([
        { type: "search" },
        { type: "search.index", args: [[]] },
        { type: "search.index.value", args: [[]] },
      ]),
    ).toBe('QueryBuilder::search().index("").value("").query()');
    expect(
      toRust([
        { type: "search" },
        { type: "search.from", args: [[]] },
        { type: "search.limit", args: [[]] },
      ]),
    ).toBe("QueryBuilder::search().from(0).limit(0).query()");
  });

  it.each<[string, CopiedStep[]]>([
    // Whole lines of Rust code around the query.
    [
      "let r = db.exec(&QueryBuilder::select().ids(1).query())?;",
      [{ type: "select" }, { type: "select.ids", args: [[f("signed", "1")]] }],
    ],
    [
      'queries.push(QueryBuilder::remove().aliases("a").query().into());',
      [
        { type: "remove" },
        { type: "remove.aliases", args: [[f("string", "a")]] },
      ],
    ],
    // Without QueryBuilder, and the short form without parentheses.
    [
      "select().node_count();",
      [{ type: "select" }, { type: "select.node_count" }],
    ],
    [".select.node_count", [{ type: "select" }, { type: "select.node_count" }]],
    // Multi-line code with comments.
    [
      `QueryBuilder::search()
        // all users
        .from("users") /* root */
        .query()`,
      [
        { type: "search" },
        { type: "search.from", args: [[f("string", "users")]] },
      ],
    ],
    // Comparison objects become the builder's convenience steps.
    [
      "search().from(1).where_().distance(CountComparison::LessThanOrEqual(2))",
      [
        { type: "search" },
        { type: "search.from", args: [[f("signed", "1")]] },
        { type: "search.where" },
        {
          type: "search.where.distance_less_than_or_equal",
          args: [[f("unsigned", "2")]],
        },
      ],
    ],
    [
      'search().from(1).where_().edge_count(Equal(2)).and().key("k").value(Comparison::Equal(1.into()))',
      [
        { type: "search" },
        { type: "search.from", args: [[f("signed", "1")]] },
        { type: "search.where" },
        { type: "search.where.edge_count", args: [[f("unsigned", "2")]] },
        { type: "search.where.and" },
        { type: "search.where.key", args: [[f("string", "k")]] },
        { type: "search.where.key.value", args: [[f("signed", "1")]] },
      ],
    ],
    [
      'search().from(1).where_().key("k").value(StartsWith("a".into()))',
      [
        { type: "search" },
        { type: "search.from", args: [[f("signed", "1")]] },
        { type: "search.where" },
        { type: "search.where.key", args: [[f("string", "k")]] },
        { type: "search.where.key.starts_with", args: [[f("string", "a")]] },
      ],
    ],
    // Literal forms: suffixes, hex, raw strings, escapes, references, wrappers.
    [
      'insert().values_uniform(&[("a", 0xFF_u64).into(), ("b", 1_000i64).into(), ("c", 2f64).into(), (r#"x"y"#, "\\u{41}\\\'\\0").into(), (String::from("d"), vec![0x10].to_owned()).into(), ("e", -3).into(), ("g", (1)).into()]).ids(vec![QueryId::from("n")])',
      [
        { type: "insert" },
        {
          type: "insert.values_uniform",
          args: [
            [f("string", "a"), f("unsigned", "255")],
            [f("string", "b"), f("signed", "1000")],
            [f("string", "c"), f("float", "2")],
            [f("string", 'x"y'), f("string", "A'\0")],
            [f("string", "d"), f("signed[]", "16")],
            [f("string", "e"), f("signed", "-3")],
            [f("string", "g"), f("signed", "1")],
          ],
        },
        { type: "insert.values.ids", args: [[f("string", "n")]] },
      ],
    ],
    [
      'insert().index("k\\\n    ey")',
      [
        { type: "insert" },
        { type: "insert.index", args: [[f("string", "key")]] },
      ],
    ],
    [
      'insert().amend_or([vec![("flags", 0x04).into()]]).ids(1)',
      [
        { type: "insert" },
        {
          type: "insert.amend_or",
          args: [[f("string", "flags"), f("signed", "4")]],
        },
        { type: "insert.values.ids", args: [[f("signed", "1")]] },
      ],
    ],
    [
      'insert().values([("k", vec![]).into()]).ids(0x1i64)',
      [
        { type: "insert" },
        {
          type: "insert.values",
          args: [[f("string", "k"), f("string[]", "")]],
        },
        { type: "insert.values.ids", args: [[f("signed", "1")]] },
      ],
    ],
    [
      "insert().index(vec!(1u32, 2))",
      [
        { type: "insert" },
        { type: "insert.index", args: [[f("unsigned[]", "1, 2")]] },
      ],
    ],
  ])("parses %s", (code, steps) => {
    expect(fromRust(code)).toEqual({ steps });
  });

  it.each([
    ["", "Expected a method"],
    ["select().node_count() extra", "Unexpected text after the query"],
    ["select().ids(1", 'Expected ")"'],
    ["select().ids(1 2)", 'Expected ")", got "2"'],
    ["select().ids(#)", 'Unexpected "#"'],
    ['select().ids("a)', "Unterminated string"],
    ['select().ids("\\q")', 'Unknown escape "\\q"'],
    ["select().ids(1.len())", "len() is not supported"],
    ["select().ids(-x)", '"-" must be followed by a number'],
    ["select().ids(--1)", '"-" must be followed by a number'],
    ["select().ids(-1_u64)", "Unsigned numbers cannot be negative"],
    ["select().ids(,)", 'Unexpected ","'],
    ["select().ids(", "Unexpected end of the query"],
    ["select().ids((1, 2, 3))", "Only (key, value) pairs"],
    [
      "QueryBuilder::select().ids(QueryBuilder::search().from(1).query())",
      "Nested queries",
    ],
    ["select().ids(foo)", 'Unsupported value "foo"'],
    ["select().ids(Some(1, 2))", "Some() must have one argument"],
    ["select().ids(DbValue::I64(1))", 'Unsupported value "DbValue::I64(…)"'],
    ["select().1", 'Unexpected "1"'],
    ["select().ids(0x_)", '"0x_" is not a number'],
    ["select().ids(0x__u64)", '"0x__u64" is not a number'],
    ['select().ids("\\u{FFFFFF}")', "is not a valid character"],
    ['select().ids("\\u{D800}")', "is not a valid character"],
  ])("rejects %s", (code, error) => {
    const result = rustLanguage.parse(code);
    expect("error" in result && result.error).toContain(error);
  });
});
