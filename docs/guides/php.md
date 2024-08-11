---
title: "PHP"
description: "PHP, Agnesoft Graph Database"
navigation:
  title: "PHP"
---

# php

The php agdb API client is generated with [openapi-generator](https://github.com/OpenAPITools/openapi-generator/blob/master/docs/generators/php.md). The following is the quickstart guide for the agdb client in PHP (connecting to the server). It assumes an `agdb_server` is running locally. Please refer to the [server guide](/docs/guides/how_to_run_server.md) to learn how to run the server.

Looking for... [how to run a server?](/docs/guides/how_to_run_server.md) | [another language?](/docs/api.md) | [embedded db guide?](/docs/guides/quickstart.md)

# Usage

The following is the from-scratch guide to use `agdb-api` php package.

<br/>1. Install [PHP](https://www.php.net/manual/en/install.php)

<br/>2. Install [Composer](https://nodejs.org/en).
<br><br>

<br/>3. Create your project's folder (e.g. `my_agdb`) and nitialize a package:
<br><br>

```bash
mkdir my_agdb
cd my_agdb
composer init # follow the steps & prompts
```

<br/>4. Add `agnesoft/agdb_api` as a dependency:
<br><br>

```bash
composer install agnesoft/agdb_api
```

NOTE: Consider using other dev packages such as `phpunit/phpunit` and `phpstan/phpstan`.

<br/>5. Create your main script (e.g. `src/index.php`) and create a client to connect to the server:
<br><br>

```php

```

<br/>7. To create a user using the default admin user:
<br><br>

```php

```

<br/>8. To create a database:
<br><br>

```php

```

<br/>9. To execute queries against the database. Notice we are feeding results of the previous query to the next one with special alias `":0"` and `":1"` referencing first and second result respectively:
<br><br>

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

<br/>10. Print the the result of the final query to the console:
<br><br>

```ts
console.log(`User (id: ${results[3].elements[0].id})`);
for (let { key, value } of results[3].elements[0].values) {
  console.log(`${key["String"]}: ${value["String"]}`);
}
```

<br/>11. Full program: https://github.com/agnesoft/agdb/tree/main/examples/server_client_php
<br><br>
