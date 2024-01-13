import { QueryBuilder } from "../src/index";
import { describe, it } from "vitest";

describe("remove", () => {
    it("remove().ids().query()", () => {
        QueryBuilder.remove().ids([1, "alias"]).query();
    });

    it("remove().aliases().query()", () => {
        QueryBuilder.remove().aliases(["alias1", "alias2"]).query();
    });

    it("remove().index().query()", () => {
        QueryBuilder.remove().index({ String: "key" }).query();
    });

    it("remove().ids().aliases().query()", () => {
        QueryBuilder.remove()
            .values([{ String: "key" }])
            .ids([1, 2])
            .query();
    });

    it("remove ids - nested queries", () => {
        QueryBuilder.remove()
            .values([{ String: "key" }])
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });

    it("remove ids - nested queries", () => {
        QueryBuilder.remove().ids(QueryBuilder.search().from(1).query()).query();
    });
});
