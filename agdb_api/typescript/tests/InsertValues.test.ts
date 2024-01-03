import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";

describe("insert values", () => {
    it("insert().values().ids().query()", () => {
        QueryBuilder.insert()
            .values([
                [
                    {
                        key: { String: "key" },
                        value: { U64: 100 },
                    },
                ],
                [],
            ])
            .ids([1, 2])
            .query();
    });

    it("nested queries", () => {
        QueryBuilder.insert()
            .values([
                [
                    {
                        key: { String: "key" },
                        value: { U64: 100 },
                    },
                ],
                [],
            ])
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });
});

describe("insert values uniform", () => {
    it("insert().values().ids().query()", () => {
        QueryBuilder.insert()
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .ids([1, 2])
            .query();
    });

    it("nested queries", () => {
        QueryBuilder.insert()
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });
});
