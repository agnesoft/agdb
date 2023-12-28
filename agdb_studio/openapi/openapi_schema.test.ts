import { QueryBuilder } from "./openapi_schema";
import { describe, it, expect, beforeEach } from "vitest";
import { components } from "./schema";

describe("agdb query", () => {
    it("insert().nodes().aliases().query()", () => {
        QueryBuilder.insert().nodes().aliases(["alias1", "alias2"]).query();
    });

    it("insert().nodes().aliases().values().query()", () => {
        QueryBuilder.insert()
            .nodes()
            .aliases(["alias1", "alias2"])
            .values([
                [
                    {
                        key: { String: "key" },
                        value: { U64: 100 },
                    },
                ],
                [],
            ])
            .query();
    });

    it("insert().nodes().aliases().values_uniform.query()", () => {
        QueryBuilder.insert()
            .nodes()
            .aliases(["alias1", "alias2"])
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .query();
    });

    it("insert().nodes().count().query()", () => {
        QueryBuilder.insert().nodes().count(1).query();
    });

    it("insert().nodes().count().values_uniform().query()", () => {
        QueryBuilder.insert()
            .nodes()
            .count(1)
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .query();
    });

    it("insert().nodes().values().query()", () => {
        QueryBuilder.insert()
            .nodes()
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

    it("insert().nodes().values_uniform().query()", () => {
        QueryBuilder.insert()
            .nodes()
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .query();
    });
});
