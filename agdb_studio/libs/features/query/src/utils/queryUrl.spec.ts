import { describe, expect, it } from "vitest";
import {
  rowsToUrlParams,
  urlParamsLength,
  sameUrlParams,
  stepsToUrlValue,
  urlParamsToRows,
  urlValueToSteps,
} from "./queryUrl";
import type { CopiedStep, QueryRow } from "../composables/types";

const ids = [[{ selectedOption: "signed", value: "1" }]];
const selectIds: CopiedStep[] = [
  { type: "select" },
  { type: "select.ids", args: ids },
];
const aliases: CopiedStep[] = [{ type: "select" }, { type: "select.aliases" }];

const row = (steps: CopiedStep[]): QueryRow => ({
  id: `row-${Math.random()}`,
  steps: steps.map((s, i) => ({ id: `s${i}`, ...s })),
});

describe("queryUrl", () => {
  it("formats steps as a short chain and reads them back", () => {
    expect(stepsToUrlValue(selectIds)).toBe("select().ids(1)");
    expect(urlValueToSteps("select().ids(1)")).toEqual(selectIds);
    // Full Rust code is accepted too.
    expect(urlValueToSteps("QueryBuilder::select().ids(1).query()")).toEqual(
      selectIds,
    );
  });

  it("round-trips steps with complex arguments", () => {
    const code =
      'insert().nodes().aliases(["a", "b"]).values_uniform([("k", 1_u64).into()])';
    const steps = urlValueToSteps(code);
    expect(steps).toBeDefined();
    expect(stepsToUrlValue(steps ?? [])).toBe(code);
  });

  it("gives undefined for values that are not a query", () => {
    expect(urlValueToSteps("")).toBeUndefined();
    expect(urlValueToSteps("select().nope()")).toBeUndefined();
  });

  it("turns rows into params, leaving out empty rows and tabs", () => {
    expect(
      rowsToUrlParams({
        exec: [row(aliases), row([]), row(selectIds)],
        exec_mut: [row([])],
        context: [row(selectIds)],
      }),
    ).toEqual({
      exec: ["select().aliases()", "select().ids(1)"],
      context: ["select().ids(1)"],
    });
  });

  it("reads rows from single, repeated and invalid params", () => {
    expect(
      urlParamsToRows({
        exec: ["select().aliases()", "nope", null, "select().ids(1)"],
        exec_mut: "nope",
        context: "select().ids(1)",
        other: "select().ids(1)",
      }),
    ).toEqual({
      rows: { exec: [aliases, selectIds], context: [selectIds] },
      // Values that are not strings are not counted.
      skipped: { exec: 1, exec_mut: 1 },
    });
    expect(urlParamsToRows({})).toEqual({ rows: {}, skipped: {} });
  });

  it("compares the route query with params", () => {
    const params = { exec: ["a", "b"], context: ["c"] };
    expect(
      sameUrlParams({ exec: ["a", "b"], context: "c", x: "1" }, params),
    ).toBe(true);
    expect(sameUrlParams({ exec: ["b", "a"], context: "c" }, params)).toBe(
      false,
    );
    expect(sameUrlParams({ exec: ["a", "b"] }, params)).toBe(false);
    expect(sameUrlParams({ exec_mut: "a" }, {})).toBe(false);
    expect(sameUrlParams({}, {})).toBe(true);
  });

  it("measures the encoded length of the rows only", () => {
    // "exec=" + "select().ids(1)" + "&"
    expect(urlParamsLength({ exec: "select().ids(1)", other: "xyz" })).toBe(21);
    expect(urlParamsLength({ context: ['a"b', "c"] })).toBe(
      "context".length * 2 + 4 + "a%22b".length + 1,
    );
    expect(urlParamsLength({})).toBe(0);
  });
});
