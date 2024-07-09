import { QueryBuilder } from "../src/index";
import { describe, it } from "vitest";

describe("insert index", () => {
    it("insert().index().query()", () => {
        QueryBuilder.insert().index({ String: "key" }).query();
    });
});
