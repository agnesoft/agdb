import { QueryBuilder } from "../src/index";
import { describe, expect, it } from "vitest";
import { AgdbApi } from "../src/index";

class MyClass {
    db_id: number | undefined | string;
    name: string = "";
    age: number = 0;
}

describe("openapi test", () => {
    it("status", async () => {
        let client = await AgdbApi.client("http://localhost:3000");
        let res = await client.status({ cluster: true });
        expect(res.status).toEqual(200);
        expect(res.data).toEqual([]);
    });

    it("insert nodes with edges", async () => {
        let client = await AgdbApi.client("http://localhost:3000");
        let admin_token = await client.user_login(null, {
            username: "admin",
            password: "admin",
        });
        AgdbApi.setToken(admin_token.data);

        await client.admin_user_add("user1", { password: "password123" });
        let token = await client.user_login(null, {
            username: "user1",
            password: "password123",
        });
        AgdbApi.setToken(token.data);

        await client.db_add({
            owner: "user1",
            db: "db1",
            db_type: "memory",
        });

        let res = await client.db_exec({ owner: "user1", db: "db1" }, [
            QueryBuilder.insert().nodes().aliases(["alias"]).query(),
            QueryBuilder.insert().nodes().count(2).query(),
        ]);

        expect(res.status).toEqual(200);

        let res2 = await client.db_exec({ owner: "user1", db: "db1" }, [
            QueryBuilder.insert()
                .edges()
                .from(["alias"])
                .to(res.data[1])
                .query(),
        ]);

        expect(res2.status).toEqual(200);
    });

    it("insert elements", async () => {
        let client = await AgdbApi.client("http://localhost:3000");
        let admin_token = await client.user_login(null, {
            username: "admin",
            password: "admin",
        });
        AgdbApi.setToken(admin_token.data);

        await client.admin_user_add("user2", { password: "password123" });
        let token = await client.user_login(null, {
            username: "user2",
            password: "password123",
        });
        AgdbApi.setToken(token.data);

        await client.db_add({
            owner: "user2",
            db: "db1",
            db_type: "memory",
        });

        let e1: MyClass = {
            db_id: 0,
            name: "John",
            age: 30,
        };

        let e2: MyClass = {
            db_id: "my_alias",
            name: "John",
            age: 30,
        };

        let e3: MyClass = {
            db_id: "my_alias",
            name: "John",
            age: 31,
        };

        let res = await client.db_exec({ owner: "user2", db: "db1" }, [
            QueryBuilder.insert().elements([e1, e2]).query(),
            QueryBuilder.insert().element(e3).query(),
        ]);

        expect(res.status).toEqual(200);

        let res2 = await client.db_exec({ owner: "user2", db: "db1" }, [
            QueryBuilder.select().ids([1, "my_alias"]).query(),
        ]);

        let expected = {
            result: 2,
            elements: [
                {
                    id: 1,
                    from: null,
                    to: null,
                    values: [
                        { key: { String: "name" }, value: { String: "John" } },
                        { key: { String: "age" }, value: { U64: 30 } },
                    ],
                },
                {
                    id: 2,
                    from: null,
                    to: null,
                    values: [
                        { key: { String: "name" }, value: { String: "John" } },
                        { key: { String: "age" }, value: { U64: 31 } },
                    ],
                },
            ],
        };

        expect(res2.status).toEqual(200);
        expect(res2.data).toEqual([expected]);
    });

    it("search elements", async () => {
        let client = await AgdbApi.client("http://localhost:3000");
        let admin_token = await client.user_login(null, {
            username: "admin",
            password: "admin",
        });
        AgdbApi.setToken(admin_token.data);

        await client.admin_user_add("user3", { password: "password123" });
        let token = await client.user_login(null, {
            username: "user3",
            password: "password123",
        });
        AgdbApi.setToken(token.data);

        await client.db_add({
            owner: "user3",
            db: "db1",
            db_type: "memory",
        });

        let res = await client.db_exec({ owner: "user3", db: "db1" }, [
            QueryBuilder.insert().nodes().count(1).query(),
            QueryBuilder.insert().nodes().count(1).query(),
            QueryBuilder.insert().edges().from([":0"]).to([":1"]).query(),
            QueryBuilder.search().elements().query(),
        ]);

        expect(res.status).toEqual(200);
        expect(res.data.length).toEqual(4);
        expect(res.data[3].result).toEqual(3);
        expect(res.data[3].elements).toEqual([
            {
                id: 1,
                from: null,
                to: null,
                values: [],
            },
            {
                id: 2,
                from: null,
                to: null,
                values: [],
            },
            {
                id: -3,
                from: 1,
                to: 2,
                values: [],
            },
        ]);
    });
});
