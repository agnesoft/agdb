import { QueryBuilder } from "../src/index";
import { describe, it, expect } from "vitest";

class MyClass {
    db_id: number | undefined | string;
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

    it("invalid db_id", () => {
        let e: MyClass = {
            db_id: "1",
            name: "John",
            age: 30,
        };
        expect(() => QueryBuilder.insert().elements([e]).query()).toThrowError(
            "invalid db_id",
        );
    });
});
