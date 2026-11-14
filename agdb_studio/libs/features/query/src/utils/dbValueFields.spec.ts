import { describe, expect, it, vi } from "vitest";
vi.mock("@agnesoft/agdb_api", async (importOriginal) => {
  return await importOriginal<typeof import("@agnesoft/agdb_api")>();
});
import type { Components } from "@agnesoft/agdb_api";
import {
  BYTES_TYPE,
  EDITABLE_VALUE_TYPES,
  NESTED_TYPE,
  dbValueToField,
  dbValueTypeLabel,
  fieldToDbValue,
  validateField,
} from "./dbValueFields";

const f = (selectedOption: string, value?: string) => ({
  selectedOption,
  value,
});

describe("dbValueFields", () => {
  it.each<[Components.Schemas.DbValue, ReturnType<typeof f>]>([
    [{ String: "hi" }, f("string", "hi")],
    // Booleans are stored as strings, so they stay strings when edited.
    [{ String: "true" }, f("string", "true")],
    [{ I64: -3 }, f("signed", "-3")],
    [{ U64: 3 }, f("unsigned", "3")],
    [{ F64: 1.5 }, f("float", "1.5")],
    [{ VecI64: [1, -2] }, f("signed[]", "1, -2")],
    [{ VecU64: [1, 2] }, f("unsigned[]", "1, 2")],
    [{ VecF64: [1.5] }, f("float[]", "1.5")],
    [{ VecString: ["a", "b"] }, f("string[]", "a, b")],
    [{ VecString: ["true", "false"] }, f("string[]", "true, false")],
    [{ VecString: [] }, f("string[]", "")],
    [{ Bytes: [1, 2] }, f(BYTES_TYPE, "1, 2")],
    [{ VecDbValue: [{ I64: 1 }, { String: "a" }] }, f(NESTED_TYPE, "1, a")],
  ])("converts %j to a field", (value, field) => {
    expect(dbValueToField(value)).toEqual(field);
  });

  it.each<Components.Schemas.DbValue>([
    { String: "hi" },
    { String: "true" },
    { I64: -3 },
    { U64: 3 },
    { F64: 1.5 },
    { F64: 2 },
    { VecI64: [1, -2] },
    { VecI64: [] },
    { VecU64: [1, 2] },
    { VecU64: [] },
    { VecF64: [1.5, 2] },
    { VecString: ["true", "false"] },
    { VecString: [] },
    { Bytes: [0, 255] },
    { Bytes: [] },
  ])("round-trips %j without changing its type", (value) => {
    const field = dbValueToField(value);
    expect(validateField(field)).toBeUndefined();
    expect(fieldToDbValue(field)).toEqual(value);
  });

  it.each<[ReturnType<typeof f>, Components.Schemas.DbValue]>([
    [f("string", "hi"), { String: "hi" }],
    [f("signed", "-3"), { I64: -3 }],
    [f("unsigned", "3"), { U64: 3 }],
    [f("float", "2"), { F64: 2 }],
    [f("boolean", "true"), { String: "true" }],
    [f("string[]", ""), { VecString: [] }],
    [f("string[]", "a, b"), { VecString: ["a", "b"] }],
    [f("boolean[]", "true"), { VecString: ["true"] }],
    [f("signed[]", "1, 2"), { VecI64: [1, 2] }],
    [f("unsigned[]", "1"), { VecU64: [1] }],
    [f("float[]", "1.5"), { VecF64: [1.5] }],
    [f(BYTES_TYPE, "1, 2"), { Bytes: [1, 2] }],
    [f(BYTES_TYPE), { Bytes: [] }],
  ])("converts field %j to a DbValue", (field, value) => {
    expect(fieldToDbValue(field)).toEqual(value);
  });

  it("offers the builder value types plus bytes", () => {
    expect(EDITABLE_VALUE_TYPES).toContain("signed");
    expect(EDITABLE_VALUE_TYPES).toContain(BYTES_TYPE);
    expect(EDITABLE_VALUE_TYPES).not.toContain(NESTED_TYPE);
  });

  it("labels value types", () => {
    expect(dbValueTypeLabel({ U64: 1 })).toBe("u64");
    expect(dbValueTypeLabel({ VecString: [] })).toBe("string[]");
    expect(
      dbValueTypeLabel({ Unknown: 1 } as unknown as Components.Schemas.DbValue),
    ).toBe("Unknown");
    expect(dbValueTypeLabel({} as Components.Schemas.DbValue)).toBe("");
  });

  const TOO_BIG = "9007199254740993"; // 2^53 + 1

  it.each<[ReturnType<typeof f>, string | undefined]>([
    [f("string", ""), undefined],
    [f("signed", "-1"), undefined],
    [f("signed", String(Number.MAX_SAFE_INTEGER)), undefined],
    [f("signed", "1.5"), "Expected an integer"],
    [f("signed", TOO_BIG), "Expected an integer"],
    [f("signed", `-${TOO_BIG}`), "Expected an integer"],
    [f("unsigned", "-1"), "Expected a non-negative integer"],
    [f("unsigned", TOO_BIG), "Expected a non-negative integer"],
    [f("unsigned"), "Expected a non-negative integer"],
    [f("float", "1.5"), undefined],
    [f("float", "-1e3"), undefined],
    [f("float", ""), "Expected a finite number"],
    [f("float", "x"), "Expected a finite number"],
    [f("float", "Infinity"), "Expected a finite number"],
    [f("float", "-Infinity"), "Expected a finite number"],
    [f("boolean", "TRUE"), undefined],
    [f("boolean", "yes"), "Expected true or false"],
    [f("signed[]", "1, -2"), undefined],
    [f("signed[]", `1, ${TOO_BIG}`), "Expected comma-separated integers"],
    [f("unsigned[]", "1, 2"), undefined],
    [f("unsigned[]", "1, x"), "Expected comma-separated non-negative"],
    [f("unsigned[]", "1, -2"), "Expected comma-separated non-negative"],
    [f("unsigned[]", TOO_BIG), "Expected comma-separated non-negative"],
    [f("float[]", "1.5, 2"), undefined],
    [f("float[]", "x"), "Expected comma-separated finite numbers"],
    [f("float[]", "1, Infinity"), "Expected comma-separated finite numbers"],
    [f("boolean[]", "true, False"), undefined],
    [f("boolean[]", "maybe"), "Expected comma-separated true/false values"],
    [f("string[]", "a"), undefined],
    [f(BYTES_TYPE, "0, 255"), undefined],
    [f(BYTES_TYPE, ""), undefined],
    [f(BYTES_TYPE, "256"), "Expected comma-separated bytes (0-255)"],
    [f(BYTES_TYPE, "-1"), "Expected comma-separated bytes (0-255)"],
    [f(BYTES_TYPE, "1.5"), "Expected comma-separated bytes (0-255)"],
    [f(NESTED_TYPE, "1, a"), undefined],
  ])("validates %j", (field, expected) => {
    const result = validateField(field);
    if (expected === undefined) {
      expect(result).toBeUndefined();
    } else {
      expect(result).toContain(expected);
    }
  });

  it("explains the precision limit for integers", () => {
    expect(validateField(f("signed", TOO_BIG))).toContain(
      String(Number.MAX_SAFE_INTEGER),
    );
  });
});
