import { QueryBuilder } from "../src/index";
import { describe, expect, it } from "vitest";
import { AgdbApi } from "../src/index";

describe("openapi test", () => {
    it("insert nodes with edges", async () => {
        let client = await AgdbApi.client("http://localhost", 3000);
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
});
