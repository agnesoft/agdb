import { QueryBuilder } from "../../src/schema/query_builder.js";
import { describe, it } from "vitest";

describe("remove", () => {
    it("remove().ids().query()", () => {
        QueryBuilder.remove().ids([1, "alias"]).query();
    });

    it("remove().aliases().query()", () => {
        QueryBuilder.remove().aliases(["alias1", "alias2"]).query();
    });

    it("remove().ids().aliases().query()", () => {
        QueryBuilder.remove()
            .values([{ String: "key" }])
            .ids([1, 2])
            .query();
    });
});
