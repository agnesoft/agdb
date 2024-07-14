import { describe, expect, it } from "vitest";
import {
    Comparison,
    convertToDbKeyValue,
    convertToDbValue,
    convertToNativeValue,
    CountComparison,
    QueryBuilder,
} from "../src/index";

import test_queries from "../../../agdb_server/openapi/test_queries.json";

class T {
    value1: string = "";
    value2: number = 0;
}

describe("QueryBuilder misc tests", () => {
    it(`convertToNativeValue`, () => {
        let bytes = convertToNativeValue({ Bytes: "1" });
        expect(typeof bytes).toStrictEqual("string");
        let truthy = convertToNativeValue({ String: "true" });
        expect(truthy).toStrictEqual(true);
        let falsey = convertToNativeValue({ String: "false" });
        expect(falsey).toStrictEqual(false);
        let u64 = convertToNativeValue({ U64: 1 });
        expect(typeof u64).toStrictEqual("number");
        let f64 = convertToNativeValue({ F64: 1.1 });
        expect(typeof f64).toStrictEqual("number");
        let vec_string = convertToNativeValue({ VecString: ["1", "2"] });
        expect(Array.isArray(vec_string)).toStrictEqual(true);
        let vec_i64 = convertToNativeValue({ VecI64: [-1, -2] });
        expect(Array.isArray(vec_i64)).toStrictEqual(true);
        let vec_u64 = convertToNativeValue({ VecU64: [1, 2] });
        expect(Array.isArray(vec_u64)).toStrictEqual(true);
        let vec_f64 = convertToNativeValue({ VecF64: [1.1, 2.2] });
        expect(Array.isArray(vec_f64)).toStrictEqual(true);
    });

    it(`convertToDbValue`, () => {
        let truthy = convertToDbValue(true);
        expect(truthy).toStrictEqual({ String: "true" });
        let falsey = convertToDbValue(false);
        expect(falsey).toStrictEqual({ String: "false" });
        let f64 = convertToDbValue(1.1);
        expect(f64).toStrictEqual({ F64: 1.1 });
        let vec_f64 = convertToDbValue([1.1, 2.2]);
        expect(vec_f64).toStrictEqual({ VecF64: [1.1, 2.2] });
        let vec_string = convertToDbValue(["hello", "world"]);
        expect(vec_string).toStrictEqual({ VecString: ["hello", "world"] });
        let nul = convertToDbValue(null);
        expect(nul).toStrictEqual(undefined);
        let undef = convertToDbValue(undefined);
        expect(undef).toStrictEqual(undefined);
        let u64 = convertToDbValue({ U64: 1 });
        expect(u64).toStrictEqual({ U64: 1 });
    });

    it(`convertToDbKeyValue`, () => {
        let kv = convertToDbKeyValue({
            key: { String: "key" },
            value: { String: "value" },
        });
        expect(kv).toStrictEqual({
            key: { String: "key" },
            value: { String: "value" },
        });
    });

    it(`CountComparison`, () => {
        let less_than = CountComparison.LessThanOrEqual(3);
        expect(less_than).toStrictEqual({ LessThanOrEqual: 3 });
    });

    it(`Comparison`, () => {
        let less_than_count = CountComparison.LessThanOrEqual(3);
        expect(less_than_count).toStrictEqual({ LessThanOrEqual: 3 });
        let greater_than = Comparison.GreaterThan(1);
        expect(greater_than).toStrictEqual({
            GreaterThan: { I64: 1 },
        });
        let greater_than_or_equal = Comparison.GreaterThanOrEqual(1);
        expect(greater_than_or_equal).toStrictEqual({
            GreaterThanOrEqual: { I64: 1 },
        });
        let less_than = Comparison.LessThan(1);
        expect(less_than).toStrictEqual({
            LessThan: { I64: 1 },
        });
        let less_than_or_equal = Comparison.LessThanOrEqual(1);
        expect(less_than_or_equal).toStrictEqual({
            LessThanOrEqual: { I64: 1 },
        });
        let not_equal = Comparison.NotEqual(1);
        expect(not_equal).toStrictEqual({
            NotEqual: { I64: 1 },
        });
    });

    it(`QueryId`, () => {
        let query = QueryBuilder.insert()
            .values([[]])
            .ids({ Alias: "alias" })
            .query();
        expect(query).toEqual({
            InsertValues: {
                ids: {
                    Ids: [
                        {
                            Alias: "alias",
                        },
                    ],
                },
                values: {
                    Multi: [[]],
                },
            },
        });
    });
});
