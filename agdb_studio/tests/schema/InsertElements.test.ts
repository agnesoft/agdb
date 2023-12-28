import { QueryBuilder } from "../../src/schema/query_builder.js";
import { describe, it } from "vitest";

class MyClass {
    db_id: number | undefined;
    name: string = "";
    age: number = 0;
}

describe("insert elements", () => {
    it("insert().element().query()", () => {
        let e: MyClass = {
            db_id: 1,
            name: "John",
            age: 30,
        };
        QueryBuilder.insert().element(e).query();
    });

    it("insert().elements().query()", () => {
        let e: MyClass = {
            db_id: 1,
            name: "John",
            age: 30,
        };
        QueryBuilder.insert().elements([e]).query();
    });
});
