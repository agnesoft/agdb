import { QueryBuilder, AgdbApi } from "agdb_api";

async function main() {
  // Requires the server to be running. Run it with `cargo run -p agdb_server`
  // from the root.

  // Creates a client connecting to the remote server.
  let client = await AgdbApi.client("http://localhost", 3000);

  // Creates a user using default admin credentials.
  let admin_token = await client.user_login(null, {
    username: "admin",
    password: "admin",
  });
  AgdbApi.setToken(admin_token.data);
  await client.admin_user_add("user1", { password: "password123" });

  // Creates a database using the newly created user.
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

  // Prepare the queries to be executed on the remote database.
  let queries = [
    QueryBuilder.insert().nodes().aliases(["users"]).query(),
    QueryBuilder.insert()
      .nodes()
      .values([
        [
          { key: { String: "username" }, value: { String: "user1" } },
          { key: { String: "password" }, value: { String: "password123" } },
        ],
        [
          { key: { String: "username" }, value: { String: "user1" } },
          { key: { String: "password" }, value: { String: "password456" } },
        ],
      ])
      .query(),
  ];

  // Execute the first batch of queries.
  let results = (await client.db_exec({ owner: "user1", db: "db1" }, queries))
    .data;

  // Prepare the second batch using the result of the previous batch.
  queries = [
    QueryBuilder.insert().edges().from(["users"]).to(results[1]).query(),
    QueryBuilder.select()
      .ids(
        QueryBuilder.search()
          .from("users")
          .where()
          .key({ String: "username" })
          .value({ Equal: { String: "user1" } })
          .query()
      )
      .query(),
  ];

  // Execute the second batch of queries.
  results = (await client.db_exec({ owner: "user1", db: "db1" }, queries)).data;

  // Print the result of the second query.
  console.log(`User (id: ${results[1].elements[0].id})`);
  for (let { key, value } of results[1].elements[0].values) {
    console.log(`${key["String"]}: ${value["String"]}`);
  }
}

main();
