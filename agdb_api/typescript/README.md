# agdb_api

This package provides client for [Agnesoft Graph Database](https://github.com/agnesoft/agdb). Please refer to the documentation of the project for details.

## Quickstart

Full guide can be found at: https://github.com/agnesoft/agdb/blob/main/agdb_web/content/en/api/2.typescript/index.md

Example project at code: https://github.com/agnesoft/agdb/tree/main/examples/server_client_typescript

```ts
import { QueryBuilder, AgdbApi } from "@agnesfot/agdb_api";

async function main() {
    // Requires the server to be running...
    // Creates a client connecting to the remote server.
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
    AgdbApi.setToken(token.data); //replaces the admin token

    // To create a database:
    await client.db_add({
        owner: "user1",
        db: "db1",
        db_type: "mapped", //memory mapped type, other options are "memory" and "file"
    });

    // Prepare and run queries:
    let queries = [
        // :0: Inserts a root node aliase "users"
        QueryBuilder.insert().nodes().aliases(["users"]).query(),
        // :1: Inserts more nodes with some data
        QueryBuilder.insert()
            .nodes()
            .values([
                [
                    { key: { String: "username" }, value: { String: "user1" } },
                    {
                        key: { String: "password" },
                        value: { String: "password123" },
                    },
                ],
                [
                    { key: { String: "username" }, value: { String: "user1" } },
                    {
                        key: { String: "password" },
                        value: { String: "password456" },
                    },
                ],
            ])
            .query(),
        // :2: Connect the root to the inserted nodes with edges referencing both from previous queries
        QueryBuilder.insert().edges().from(":0").to(":1").query(),

        // :3: Find a node starting at the "users" node (could also be ":0" in this instance) with specific username
        QueryBuilder.select()
            .ids(
                QueryBuilder.search()
                    .from("users")
                    .where()
                    .key({ String: "username" })
                    .value({ Equal: { String: "user1" } })
                    .query(),
            )
            .query(),
    ];
    // Execute queries.
    let results = (await client.db_exec({ owner: "user1", db: "db1" }, queries))
        .data;
    console.log(`User (id: ${results[3].elements[0].id})`);
    for (let { key, value } of results[3].elements[0].values) {
        console.log(`${key["String"]}: ${value["String"]}`);
    }
}
```
