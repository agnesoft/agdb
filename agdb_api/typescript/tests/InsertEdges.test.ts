import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";

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
});
