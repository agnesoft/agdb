import { QueryBuilder } from "../src/index";
import { describe, it } from "vitest";
import { client } from "./test_setup";

describe("insert nodes", () => {
    it("insert().nodes().aliases().query()", async () => {
        await client.db_add({
            owner: "admin",
            db: "insert_nodes",
            db_type: "memory",
        });
        client.db_exec({ owner: "admin", db: "insert_nodes" }, [
            QueryBuilder.insert().nodes().aliases(["alias1", "alias2"]).query(),
        ]);

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

    it("insert().nodes().ids().count().query()", () => {
        QueryBuilder.insert().nodes().ids(["alias"]).count(1).query();
    });

    it("insert().nodes().ids().aliases().query()", () => {
        QueryBuilder.insert().nodes().ids(["alias"]).aliases(["alias"]).query();
    });

    it("insert().nodes().ids().values_uniform().query()", () => {
        QueryBuilder.insert()
            .nodes()
            .ids(["alias"])
            .values_uniform([
                {
                    key: { String: "key" },
                    value: { U64: 100 },
                },
            ])
            .query();
    });

    it("insert().nodes().ids().values().query()", () => {
        QueryBuilder.insert()
            .nodes()
            .ids(["alias"])
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
});
