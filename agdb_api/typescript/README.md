# agdb_api

This package provides the client, generated from the Rust one, for [Agnesoft Graph Database](https://github.com/agnesoft/agdb). Please refer to the documentation of the project for details.

## Quickstart

Full guide can be found at: https://github.com/agnesoft/agdb/blob/main/agdb_web/content/en/api/2.typescript/index.md

Example project at code: https://github.com/agnesoft/agdb/tree/main/agdb_api/typescript/examples/server_client

```ts
import { AgdbApi, FetchClient, QueryBuilder } from "@agnesoft/agdb_api";

async function main() {
    // Requires the server to be running...
    // Creates a client connecting to the remote server.
    const client = AgdbApi.new(new FetchClient(), "localhost:3000");
    await client.user_login({ username: "admin", password: "admin" });
    await client.admin_user_add({ username: "user1", password: "password123" });
    await client.user_login({ username: "user1", password: "password123" }); //replaces the admin token

    // To create a database:
    await client.db_add({ owner: "user1", db: "db1", db_type: "mapped" }); //memory mapped type, other options are "memory" and "file"

    // Prepare and run queries:
    const queries = [
        // :0: Inserts a root node aliased "users"
        QueryBuilder.insert().nodes().aliases(["users"]).query(),
        // :1: Inserts more nodes with some data
        QueryBuilder.insert()
            .nodes()
            .values([
                { username: "user1", password: "password123" },
                { username: "user2", password: "password456" },
            ])
            .query(),
        // :2: Connect the root to the inserted nodes with edges referencing both from previous queries
        QueryBuilder.insert().edges().from(":0").to(":1").query(),

        // :3: Find a node starting at the "users" node (could also be ":0" in this instance) with specific username
        QueryBuilder.select()
            .search()
            .from("users")
            .where()
            .key("username")
            .value("user1")
            .query(),
    ];
    // Execute queries.
    const results = (
        await client.db_exec_mut({ owner: "user1", db: "db1", queries })
    )[1];
    console.log(`User (id: ${results[3].elements[0].id})`);
    for (const { key, value } of results[3].elements[0].values) {
        console.log(`${JSON.stringify(key)}: ${JSON.stringify(value)}`);
    }
}
```
