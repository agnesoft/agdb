import { QueryBuilder } from "../src/index";
import { describe, it } from "vitest";

describe("select", () => {
    it("select().aliases().query()", () => {
        QueryBuilder.select().aliases().query();
    });

    it("select().aliases().ids().query()", () => {
        QueryBuilder.select().aliases().ids([1, 2]).query();
    });

    it("select().edge_count().ids().query()", () => {
        QueryBuilder.select().edge_count().ids([1, 2]).query();
    });

    it("select().edge_count_from().ids().query()", () => {
        QueryBuilder.select().edge_count_from().ids([1, 2]).query();
    });

    it("select().edge_count_to().ids().query()", () => {
        QueryBuilder.select().edge_count_to().ids([1, 2]).query();
    });

    it("select().indexes().query()", () => {
        QueryBuilder.select().indexes().query();
    });

    it("select().ids().query()", () => {
        QueryBuilder.select().ids([1, 2]).query();
    });

    it("select().keys().ids().query()", () => {
        QueryBuilder.select().keys().ids([1, 2]).query();
    });

    it("select().key_counts().ids().query()", () => {
        QueryBuilder.select().key_count().ids([1, 2]).query();
    });

    it("select().values().ids().query()", () => {
        QueryBuilder.select()
            .values([{ String: "key" }, { U64: 100 }])
            .ids([1, 2])
            .query();
    });

    it("select aliases - nested queries", () => {
        QueryBuilder.select()
            .aliases()
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });

    it("select ids - nested queries", () => {
        QueryBuilder.select()
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });

    it("select keys - nested queries", () => {
        QueryBuilder.select()
            .keys()
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });

    it("select values - nested queries", () => {
        QueryBuilder.select()
            .values([{ String: "key" }, { U64: 100 }])
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });

    it("select key count - nested queries", () => {
        QueryBuilder.select()
            .key_count()
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });

    it("select edge count - nested queries", () => {
        QueryBuilder.select()
            .edge_count()
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });
});
