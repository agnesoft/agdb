import { QueryBuilder } from "../../src/schema/query_builder.js";
import { describe, it } from "vitest";

describe("insert aliases", () => {
    it("insert().aliases().ids().query()", () => {
        QueryBuilder.insert().aliases(["alias1", "alias2"]).ids([1, 2]).query();
    });
});
