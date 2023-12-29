import { QueryBuilder } from "../src/query_builder";
import { describe, it } from "vitest";
import { Api } from "./client";

describe("openapi test", () => {
    it("insert node", async () => {
        let client = await Api.client();
        let admin_token = await client.paths["/api/v1/admin/login"].post({ password: "admin" });
        Api.setToken(admin_token);

        client.paths["/api/v1/admin/user/{username}/add"].post("user1", {
            password: "password123",
        });

        let token = await client.paths["/api/v1/{username}/login"].post({
            password: "password123",
        });

        Api.setToken(token);
        let query = QueryBuilder.insert().aliases(["alias1", "alias2"]).ids([1, 2]).query();

        client.paths["/api/v1/db/{owner}/{db}/exec"].post({ owner: "user1", db: "db1" }, [query]);
    });
});
