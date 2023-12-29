import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";
import { Api } from "./client";

describe("openapi test", () => {
    it("insert node", async () => {
        let client = await Api.client();
        let admin_token = await client.user_login("admin", { password: "admin" });
        Api.setToken(admin_token.data);

        await client.admin_user_add("user1", { password: "password123" });
        let token = await client.user_login("user1", { password: "password123" });
        Api.setToken(token.data);

        await client.db_add({ owner: "user1", db: "db1", db_type: "memory" });

        let query = QueryBuilder.insert().aliases(["alias1", "alias2"]).ids([1, 2]).query();
        await client.db_exec({ owner: "user1", db: "db1" }, [query]);
    });
});
