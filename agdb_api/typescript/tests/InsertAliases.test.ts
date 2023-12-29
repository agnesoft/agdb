import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";
import { Api } from "./client";

describe("insert aliases", () => {
    it("insert().aliases().ids().query()", () => {
        QueryBuilder.insert().aliases(["alias1", "alias2"]).ids([1, 2]).query();
    });
});
