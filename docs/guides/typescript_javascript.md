---
title: "Typescript, Javascript"
description: "Typescript, Javascript, Agnesoft Graph Database"
navigation:
  title: "Typescript, Javascript"
---

# typescript / javascript

The typescript agdb API client is based on [openapi-client-axios](https://www.npmjs.com/package/openapi-client-axios). The following is the quickstart guide for the agdb client in Javascript/Typescript (connecting to the server). It assumes an `agdb_server` is running locally. Please refer to the [server guide](/docs/guides/how_to_run_server.md) to learn how to run the server.

Looking for... [how to run a server?](/docs/guides/how_to_run_server.md) | [another language?](/docs/api.md) | [embedded db guide?](/docs/guides/quickstart.md)

# Usage

The following is the from-scratch guide to use `agdb-api` typescript/javascript package.

### 1. Install [NodeJS](https://nodejs.org/en).

### 2. Create your project's folder (e.g. `my_agdb`) and nitialize a package:

```bash
mkdir my_agdb
cd my_agdb
npm init # follow the steps & prompts
```

### 3. Add `typescript` as your dev dependency:

```bash
npm install typescript --save-dev
```

NOTE: Consider using other dev packages such as `prettier`, `eslint` (and `@typescript-eslint/parser`).

### 4. Add `@agnesoft/agdb_api` npm package to your project:

```bash
npm install @agnesoft/agdb_api
```

### 5. Create a `tsconfig.json` file:

```json
{
  "compilerOptions": {
    "module": "ESNext",
    "sourceMap": true,
    "lib": ["ES2015", "DOM"],
    "moduleResolution": "node",
    "allowJs": true,
    "esModuleInterop": true
  }
}
```

### 6. In your main script (`indes.ts` or `main.ts` depending on your `package.json`'s `"main"` field) create a client connecting to the server:

```ts
import { QueryBuilder, Comparison, AgdbApi } from "@agnesfot/agdb_api";

async function main() {
  // Requires the server to be running...

  // Creates a client connecting to the remote server.
  let client = await AgdbApi.client("http://localhost:3000");
}
```

### 7. To create a user using the default admin user:

```ts
await client.login("admin", "admin");
await client.admin_user_add("user1", { password: "password123" });
await client.login("user1", "password123");
```

### 8. To create a database:

```ts
await client.db_add({
  owner: "user1",
  db: "db1",
  db_type: "mapped", //memory mapped type, other options are "memory" and "file"
});
```

### 9. To execute queries against the database. Notice we are feeding results of the previous query to the next one with special alias `":0"` and `":1"` referencing first and second result respectively:

```ts
// Prepare the queries to be executed on the remote database.
let queries = [
  // :0: Inserts a root node aliase "users"
  QueryBuilder.insert().nodes().aliases(["users"]).query(),

  // :1: Inserts more nodes with some data
  QueryBuilder.insert()
    .nodes()
    .values([
      [
        ["username", "user1"],
        ["password", "password123"],
      ],
      [
        ["username", "user1"],
        ["password", "password456"],
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
        .key("username")
        .value(Comparison.Equal("user1"))
        .query()
    )
    .query(),
];

// Execute queries.
let results = (await client.db_exec({ owner: "user1", db: "db1" }, queries))
  .data;
```

### 10. Print the the result of the final query to the console:

```ts
console.log(`User (id: ${results[3].elements[0].id})`);
for (let { key, value } of results[3].elements[0].values) {
  console.log(`${key["String"]}: ${value["String"]}`);
}
```

### 11. See full program in the examples: https://github.com/agnesoft/agdb/tree/main/examples/server_client_typescript
