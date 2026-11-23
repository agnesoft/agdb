import { AgdbApi, FetchClient, QueryBuilder } from "@agnesoft/agdb_api";

async function main() {
    // Requires the server to be running. Run it with `cargo run -p agdb_server`
    // from the root.

    const client = AgdbApi.new(new FetchClient(), "localhost:3000");

    // Log in as admin and create a new user.
    await client.user_login({ username: "admin", password: "admin" });
    await client.admin_user_add({
        username: "client",
        password: "password111",
    });

    // Switch to the new user and create a database.
    await client.user_login({ username: "client", password: "password111" });
    await client.db_add({ owner: "client", db: "db", db_type: "memory" });

    const users = [
        { username: "user1", password: "password123" },
        { username: "user2", password: "password456" },
    ];

    // Insert a root "users" alias and the user nodes.
    let queries = [
        QueryBuilder.insert().nodes().aliases(["users"]).query(),
        QueryBuilder.insert().nodes().values(users).query(),
    ];
    let results = (
        await client.db_exec_mut({ owner: "client", db: "db", queries })
    )[1];

    // Link users to root and search for one by username.
    queries = [
        QueryBuilder.insert().edges().from("users").to(results[1]).query(),
        QueryBuilder.select()
            .search()
            .depth_first()
            .from("users")
            .where()
            .key("username")
            .value("user1")
            .query(),
    ];
    results = (
        await client.db_exec_mut({ owner: "client", db: "db", queries })
    )[1];

    console.log(`User: ${results[1].elements[0].id}`);
    for (const { key, value } of results[1].elements[0].values) {
        console.log(`  ${JSON.stringify(key)}: ${JSON.stringify(value)}`);
    }
}

main();
