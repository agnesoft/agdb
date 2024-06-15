import { QueryBuilder } from "../src/index";
import { describe, it, expect } from "vitest";

class MyClass {
    db_id: number | undefined | string;
    name: string = "";
    age: number = 0;
}

class BadClass {
    db_id: any;
    name: any = "";
    age: number = 0;
}

class MyClassWithTypes {
    db_id: number | undefined | string;
    truthy: boolean = true;
    truthy2: boolean = false;
    i64: number = -1;
    u64: number = 1;
    f64: number = 1.1;
    vec_i64: number[] = [-1, 0, 1];
    vec_u64: number[] = [1, 2, 3];
    vec_f64: number[] = [1.1, 2.2, 3.3];
    vec_string: string[] = ["a", "b", "c"];
    vec_bool: boolean[] = [true, false, true];
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
        let e: BadClass = {
            db_id: {},
            name: "John",
            age: 30,
        };
        expect(() => QueryBuilder.insert().elements([e]).query()).toThrowError(
            "invalid db_id",
        );
    });

    it("insert().element().query()", () => {
        let e: MyClassWithTypes = {
            db_id: 1,
            truthy: true,
            truthy2: false,
            i64: -1,
            u64: 1,
            f64: 1.1,
            vec_i64: [-1, 0, 1],
            vec_u64: [1, 2, 3],
            vec_f64: [1.1, 2.2, 3.3],
            vec_string: ["a", "b", "c"],
            vec_bool: [true, false, true],
        };
        QueryBuilder.insert().element(e).query();
    });

    it("invalid type", () => {
        let e: BadClass = {
            db_id: undefined,
            name: undefined,
            age: 30,
        };
        QueryBuilder.insert().elements([e]).query();
    });

    it("unsupported type", () => {
        let e: BadClass = {
            db_id: undefined,
            name: {},
            age: 30,
        };
        expect(() => QueryBuilder.insert().elements([e]).query()).toThrowError(
            "Unsupported type for DbValue conversion: object",
        );
    });
});
