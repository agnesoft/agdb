import { describe, expect, it } from "vitest";
import {
    convertTo,
    convertToDbKeyValue,
    convertToDbValue,
    convertToNativeValue,
} from "../src/index.ts";
import type { QueryResult } from "../src/index.ts";

describe("QueryBuilder misc tests", () => {
    it(`convertToNativeValue`, () => {
        const bytes = convertToNativeValue({ Bytes: [1] });
        expect(typeof bytes).toStrictEqual("object");
        const truthy = convertToNativeValue({ String: "true" });
        expect(truthy).toStrictEqual(true);
        const falsey = convertToNativeValue({ String: "false" });
        expect(falsey).toStrictEqual(false);
        const i64 = convertToNativeValue({ I64: -1 });
        expect(i64).toStrictEqual(-1);
        const string = convertToNativeValue({ String: "s" });
        expect(string).toStrictEqual("s");
        const u64 = convertToNativeValue({ U64: 1 });
        expect(typeof u64).toStrictEqual("number");
        const f64 = convertToNativeValue({ F64: 1.1 });
        expect(typeof f64).toStrictEqual("number");
        const vec_string = convertToNativeValue({ VecString: ["1", "2"] });
        expect(Array.isArray(vec_string)).toStrictEqual(true);
        const vec_i64 = convertToNativeValue({ VecI64: [-1, -2] });
        expect(Array.isArray(vec_i64)).toStrictEqual(true);
        const vec_u64 = convertToNativeValue({ VecU64: [1, 2] });
        expect(Array.isArray(vec_u64)).toStrictEqual(true);
        const vec_f64 = convertToNativeValue({ VecF64: [1.1, 2.2] });
        expect(Array.isArray(vec_f64)).toStrictEqual(true);
        const none = convertToNativeValue("None");
        expect(none).toStrictEqual(undefined);
    });

    it(`convertToDbValue`, () => {
        const truthy = convertToDbValue(true);
        expect(truthy).toStrictEqual({ String: "true" });
        const falsey = convertToDbValue(false);
        expect(falsey).toStrictEqual({ String: "false" });
        const string = convertToDbValue("s");
        expect(string).toStrictEqual({ String: "s" });
        const i64 = convertToDbValue(1);
        expect(i64).toStrictEqual({ I64: 1 });
        const vec_i64 = convertToDbValue([1, 2]);
        expect(vec_i64).toStrictEqual({ VecI64: [1, 2] });
        const f64 = convertToDbValue(1.1);
        expect(f64).toStrictEqual({ F64: 1.1 });
        const vec_f64 = convertToDbValue([1.1, 2.2]);
        expect(vec_f64).toStrictEqual({ VecF64: [1.1, 2.2] });
        const vec_string = convertToDbValue(["hello", "world"]);
        expect(vec_string).toStrictEqual({ VecString: ["hello", "world"] });
        const u64 = convertToDbValue({ U64: 1 });
        expect(u64).toStrictEqual({ U64: 1 });
    });

    it(`convertToDbKeyValue`, () => {
        const pair = convertToDbKeyValue(["key", 1]);
        expect(pair).toStrictEqual({
            key: { String: "key" },
            value: { I64: 1 },
        });
        const kv = convertToDbKeyValue({
            key: { String: "key" },
            value: { String: "value" },
        });
        expect(kv).toStrictEqual({
            key: { String: "key" },
            value: { String: "value" },
        });
    });

    it(`convertTo single element`, () => {
        type User = { db_id: number; name: string };
        const result: QueryResult = {
            result: 1,
            elements: [
                {
                    id: 1,
                    from: 0,
                    to: 0,
                    values: [
                        {
                            key: { String: "name" },
                            value: { String: "alice" },
                        },
                    ],
                },
            ],
        };
        const user = convertTo<User>(result);
        expect(user).toEqual({ db_id: 1, name: "alice" });
    });

    it(`convertTo multiple elements`, () => {
        type User = { db_id: number; name: string };
        const result: QueryResult = {
            result: 2,
            elements: [
                {
                    id: 1,
                    from: 0,
                    to: 0,
                    values: [
                        {
                            key: { String: "name" },
                            value: { String: "alice" },
                        },
                    ],
                },
                {
                    id: 2,
                    from: 0,
                    to: 0,
                    values: [
                        {
                            key: { String: "name" },
                            value: { String: "bob" },
                        },
                    ],
                },
            ],
        };
        const users = convertTo<User>(result);
        expect(users).toEqual([
            { db_id: 1, name: "alice" },
            { db_id: 2, name: "bob" },
        ]);
    });
});
