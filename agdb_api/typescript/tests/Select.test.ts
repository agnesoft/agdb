import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";

describe("select", () => {
    it("select().aliases().query()", () => {
        QueryBuilder.select().aliases().query();
    });

    it("select().aliases().ids().query()", () => {
        QueryBuilder.select().aliases().ids([1, 2]).query();
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
});
