import { QueryBuilder } from "../src/index";
import { describe, it, expect } from "vitest";

describe("insert edges", () => {
    it("insert().edges().from().to().query()", () => {
        QueryBuilder.insert().edges().from([1]).to([2]).query();
    });

    it("insert().edges().from().to().values().query()", () => {
        QueryBuilder.insert()
            .edges()
            .from([1])
            .to([2])
            .values([
                [
                    {
                        key: { String: "key" },
                        value: { U64: 100 },
                    },
                ],
            ])
            .query();
    });

    it("insert().edges().from().to().values_uniform().query()", () => {
        QueryBuilder.insert()
            .edges()
            .from([1])
            .to([2])
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .query();
    });

    it("insert().edges().from().to().each().query()", () => {
        QueryBuilder.insert().edges().from([1]).to([2]).each().query();
    });

    it("insert().edges().from().to().each().values().query()", () => {
        QueryBuilder.insert()
            .edges()
            .from([1])
            .to([2])
            .each()
            .values([
                [
                    {
                        key: { String: "key" },
                        value: { U64: 100 },
                    },
                ],
            ])
            .query();
    });

    it("insert().edges().from().to().each().values_uniform().query()", () => {
        QueryBuilder.insert()
            .edges()
            .from([1])
            .to([2])
            .each()
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .query();
    });

    it("nested queries", () => {
        QueryBuilder.insert()
            .edges()
            .from(QueryBuilder.search().from(1).query())
            .to(QueryBuilder.search().from(2).query())
            .query();
    });

    it("invalid nested queries", () => {
        expect(() =>
            QueryBuilder.insert()
                .edges()
                .from(QueryBuilder.insert().nodes().count(1).query())
                .to(QueryBuilder.search().from(2).query())
                .query(),
        ).toThrowError("invalid search query");
    });
});
