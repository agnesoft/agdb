import { describe, expect, it } from "vitest";
import {
    Comparison,
    Components,
    convertToDbKeyValue,
    convertToDbValue,
    convertToNativeValue,
    CountComparison,
    DbKeyOrder,
    QueryBuilder,
} from "../src/index";

describe("QueryBuilder misc tests", () => {
    it(`convertToNativeValue`, () => {
        const bytes = convertToNativeValue({ Bytes: [1] });
        expect(typeof bytes).toStrictEqual("object");
        const truthy = convertToNativeValue({ String: "true" });
        expect(truthy).toStrictEqual(true);
        const falsey = convertToNativeValue({ String: "false" });
        expect(falsey).toStrictEqual(false);
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
    });

    it(`convertToDbValue`, () => {
        const truthy = convertToDbValue(true);
        expect(truthy).toStrictEqual({ String: "true" });
        const falsey = convertToDbValue(false);
        expect(falsey).toStrictEqual({ String: "false" });
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
        const kv = convertToDbKeyValue({
            key: { String: "key" },
            value: { String: "value" },
        });
        expect(kv).toStrictEqual({
            key: { String: "key" },
            value: { String: "value" },
        });
    });

    it(`CountComparison`, () => {
        const less_than = CountComparison.LessThanOrEqual(3);
        expect(less_than).toStrictEqual({ LessThanOrEqual: 3 });
    });

    it(`Comparison`, () => {
        const less_than_count = CountComparison.LessThanOrEqual(3);
        expect(less_than_count).toStrictEqual({ LessThanOrEqual: 3 });
        const greater_than = Comparison.GreaterThan(1);
        expect(greater_than).toStrictEqual({
            GreaterThan: { I64: 1 },
        });
        const greater_than_or_equal = Comparison.GreaterThanOrEqual(1);
        expect(greater_than_or_equal).toStrictEqual({
            GreaterThanOrEqual: { I64: 1 },
        });
        const less_than = Comparison.LessThan(1);
        expect(less_than).toStrictEqual({
            LessThan: { I64: 1 },
        });
        const less_than_or_equal = Comparison.LessThanOrEqual(1);
        expect(less_than_or_equal).toStrictEqual({
            LessThanOrEqual: { I64: 1 },
        });
        const not_equal = Comparison.NotEqual(1);
        expect(not_equal).toStrictEqual({
            NotEqual: { I64: 1 },
        });
    });

    it(`QueryId`, () => {
        const query = QueryBuilder.insert()
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

    it(`db_id as QueryId`, () => {
        class T {
            db_id: Components.Schemas.QueryId = { Alias: "alias" };
        }

        const query = QueryBuilder.insert().element(new T()).query();
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

    it(`db_id invalid type`, () => {
        class T {
            db_id: string[] = ["hello"];
        }

        expect(() => QueryBuilder.insert().element(new T()).query()).toThrow(
            "Invalid db_id type",
        );
    });

    it("single order_by", () => {
        const query = QueryBuilder.search()
            .from(1)
            .order_by(DbKeyOrder.Asc("key"))
            .query();
        const expected = {
            Search: {
                algorithm: "BreadthFirst",
                origin: {
                    Id: 1,
                },
                destination: {
                    Id: 0,
                },
                limit: 0,
                offset: 0,
                order_by: [{ Asc: { String: "key" } }],
                conditions: [],
            },
        };
        expect(query).toEqual(expected);
    });

    it("shorthand equal comparisons", () => {
        const distance = QueryBuilder.search()
            .from(1)
            .where()
            .distance(2)
            .query();
        expect(distance["Search"].conditions[0].data).toEqual({
            Distance: { Equal: 2 },
        });

        const edge_count = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count(2)
            .query();
        expect(edge_count["Search"].conditions[0].data).toEqual({
            EdgeCount: { Equal: 2 },
        });

        const edge_count_from = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count_from(2)
            .query();
        expect(edge_count_from["Search"].conditions[0].data).toEqual({
            EdgeCountFrom: { Equal: 2 },
        });

        const edge_count_to = QueryBuilder.search()
            .from(1)
            .where()
            .edge_count_to(2)
            .query();
        expect(edge_count_to["Search"].conditions[0].data).toEqual({
            EdgeCountTo: { Equal: 2 },
        });
    });
});
