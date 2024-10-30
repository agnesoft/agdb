import { convertTo, QueryBuilder } from "../src/index";
import { describe, expect, it } from "vitest";
import { AgdbApi } from "../src/index";

class MyClass {
    db_id: number | undefined | string;
    name: string = "";
    age: number = 0;
}

describe("openapi test", () => {
    it("status", async () => {
        const client = await AgdbApi.client("http://localhost:3000");
        const res = await client.status();
        expect(res.status).toEqual(200);
    });

    it("logout", async () => {
        const client = await AgdbApi.client("http://localhost:3000");
        await client.login("admin", "admin");
        await client.logout();
        expect(client.get_token()).toEqual(undefined);

        await expect(client.db_list()).rejects.toThrowError(
            "Request failed with status code 401",
        );
    });

    it("manual token", async () => {
        const client = await AgdbApi.client("http://localhost:3000");
        const token = await client.user_login(null, {
            username: "admin",
            password: "admin",
        });
        client.set_token(token.data);
        expect(client.get_token()).toEqual(token.data);
        expect((await client.db_list()).status).toBe(200);
    });

    it("insert nodes with edges", async () => {
        const admin_client = await AgdbApi.client("http://localhost:3000");
        await admin_client.login("admin", "admin");
        await admin_client.admin_user_add("user1", { password: "password123" });

        const client = await AgdbApi.client("http://localhost:3000");
        await client.login("user1", "password123");
        await client.db_add({
            owner: "user1",
            db: "db1",
            db_type: "memory",
        });

        const res = await client.db_exec({ owner: "user1", db: "db1" }, [
            QueryBuilder.insert().nodes().aliases("alias").query(),
            QueryBuilder.insert().nodes().count(2).query(),
        ]);

        expect(res.status).toEqual(200);

        const res2 = await client.db_exec({ owner: "user1", db: "db1" }, [
            QueryBuilder.insert().edges().from("alias").to(res.data[1]).query(),
        ]);

        expect(res2.status).toEqual(200);
    });

    it("insert elements", async () => {
        const admin_client = await AgdbApi.client("http://localhost:3000");
        await admin_client.login("admin", "admin");
        await admin_client.admin_user_add("user2", { password: "password123" });

        const client = await AgdbApi.client("http://localhost:3000");
        await client.login("user2", "password123");
        await client.db_add({
            owner: "user2",
            db: "db1",
            db_type: "memory",
        });

        const e1: MyClass = {
            db_id: 0,
            name: "John",
            age: 30,
        };

        const e2: MyClass = {
            db_id: "my_alias",
            name: "John",
            age: 30,
        };

        const e3: MyClass = {
            db_id: "my_alias",
            name: "John",
            age: 31,
        };

        const res = await client.db_exec({ owner: "user2", db: "db1" }, [
            QueryBuilder.insert().elements([e1, e2]).query(),
            QueryBuilder.insert().element(e3).query(),
        ]);

        expect(res.status).toEqual(200);

        const res2 = await client.db_exec({ owner: "user2", db: "db1" }, [
            QueryBuilder.select().ids([1, "my_alias"]).query(),
            QueryBuilder.select().ids("my_alias").query(),
        ]);

        const expected = {
            result: 2,
            elements: [
                {
                    id: 1,
                    from: null,
                    to: null,
                    values: [
                        { key: { String: "name" }, value: { String: "John" } },
                        { key: { String: "age" }, value: { I64: 30 } },
                    ],
                },
                {
                    id: 2,
                    from: null,
                    to: null,
                    values: [
                        { key: { String: "name" }, value: { String: "John" } },
                        { key: { String: "age" }, value: { I64: 31 } },
                    ],
                },
            ],
        };

        expect(res2.status).toEqual(200);
        expect(res2.data.length).toEqual(2);
        expect(res2.data[0]).toEqual(expected);

        const expected1: MyClass = {
            db_id: 1,
            name: "John",
            age: 30,
        };

        const expected2: MyClass = {
            db_id: 2,
            name: "John",
            age: 31,
        };

        let list = convertTo<MyClass>(res2.data[0]);
        expect(list).toEqual([expected1, expected2]);
        list = convertTo<MyClass>(res2.data[1]);
        expect(list).toEqual(expected2);
    });

    it("search elements", async () => {
        const admin_client = await AgdbApi.client("http://localhost:3000");
        await admin_client.login("admin", "admin");
        await admin_client.admin_user_add("user3", { password: "password123" });

        const client = await AgdbApi.client("http://localhost:3000");
        await client.login("user3", "password123");
        await client.db_add({
            owner: "user3",
            db: "db1",
            db_type: "memory",
        });

        const res = await client.db_exec({ owner: "user3", db: "db1" }, [
            QueryBuilder.insert().nodes().count(1).query(),
            QueryBuilder.insert().nodes().count(1).query(),
            QueryBuilder.insert().edges().from(":0").to(":1").query(),
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
