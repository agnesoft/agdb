import { QueryBuilder } from "../src/query_builder";
import { describe, expect, it } from "vitest";
import { Api } from "./client";

describe("openapi test", () => {
    it("insert node", async () => {
        let client = await Api.client();
        let admin_token = await client.user_login(null, { username: "admin", password: "admin" });
        Api.setToken(admin_token.data);

        await client.admin_user_add("user1", { password: "password123" });
        let token = await client.user_login(null, { username: "user1", password: "password123" });
        Api.setToken(token.data);

        await client.db_add({
            owner: "user1",
            db: "db1",
            db_type: "memory",
        });

        let query1 = QueryBuilder.insert().nodes().count(2).query();
        let query2 = QueryBuilder.insert().aliases(["alias1", "alias2"]).ids([1, 2]).query();
        let res = await client.db_exec({ owner: "user1", db: "db1" }, [query1, query2]);

        expect(res.status).toEqual(200);
    });
});
