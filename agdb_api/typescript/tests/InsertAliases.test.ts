import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";

describe("insert aliases", () => {
    it("insert().aliases().ids().query()", () => {
        QueryBuilder.insert().aliases(["alias1", "alias2"]).ids([1, 2]).query();
    });

    it("nested queries", () => {
        QueryBuilder.insert()
            .aliases(["alias1", "alias2"])
            .ids(QueryBuilder.search().from(1).query())
            .query();
    });
});
