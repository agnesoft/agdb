# Agnesoft Graph Database

[![Crates.io](https://img.shields.io/crates/v/agdb)](https://crates.io/crates/agdb) [![release](https://github.com/agnesoft/agdb/actions/workflows/release.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/release.yaml) [![coverage](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml) [![codecov](https://codecov.io/gh/agnesoft/agdb/branch/main/graph/badge.svg?token=Z6YO8C3XGU)](https://codecov.io/gh/agnesoft/agdb)

The Agnesoft Graph Database (aka _agdb_) is persistent memory mapped graph database using object 'no-text' queries. It can be used as a main persistent storage, data analytics platform as well as fast in-memory cache. Its typed schema-less data store allows for flexible and seamless data updates with no downtime or costly migrations. All queries are constructed via a builder pattern (or directly as objects) with no special language or text parsing.

- [Agnesoft Graph Database](#agnesoft-graph-database)
- [Key Features](#key-features)
- [Quickstart](#quickstart)
- [Roadmap](#roadmap)
- [Reference](#reference)
  - [Efficient agdb](#efficient-agdb)
  - [Concepts](#concepts)
  - [Queries](#queries)
  - [But why?](#but-why)

# Key Features

- Data plotted on a graph
- Typed [key-value properties](docs/concepts.md#data-types) attached to graph elements (nodes & edges)
- Persistent file based storage
- ACID compliant
- [Object queries](docs/queries.md) with builder pattern (no text, no query language)
- Memory mapped for fast querying

# Quickstart

```
cargo add agdb
```

Basic usage demonstrating creating a database, inserting graph elements with data and querying them back with select and search. The function using this code must handle `agdb::DbError` and [`agdb::QueryError`](docs/queries.md#queryerror) error types for operator `?` to work:

```Rust
use agdb::{Db, DbId, QueryBuilder, UserValue, DbUserValue, Comparison::Equal};

let mut db = Db::new("db_file.agdb")?;

db.exec_mut(&QueryBuilder::insert().nodes().aliases("users").query())?;

#[derive(Debug, UserValue)]
struct User { db_id: Option<DbId>, name: String, }
let users = vec![User { db_id: None, name: "Alice".to_string(), },
                 User { db_id: None, name: "Bob".to_string(), },
                 User { db_id: None, name: "John".to_string(), }];

let users_ids = db.exec_mut(&QueryBuilder::insert().nodes().values(&users).query())?;

db.exec_mut(
    &QueryBuilder::insert()
        .edges()
        .from("users")
        .to(&users_ids)
        .query(),
)?;
```

This code creates a database called `user_db.agdb` with a simple graph of 4 nodes. The first node is aliased `users` and 3 user nodes for Alice, Bob and John are then connected with edges to the `users` node. The arbitrary `name` property is attached to the user nodes. Rather than inserting values directly with keys (which is also possible) we use our own type and derive from `agdb::UserValue` to allow it to be used with the database.

You can select the graph elements (both nodes & edges) with their ids to get them back with their associated data (key-value properties). Lets select our users and convert the result into the list (notice we select only values relevant to our `User` type with passing `User::db_keys()`):

```Rust
let users: Vec<User> = db
    .exec(
        &QueryBuilder::select()
            .values(User::db_keys())
            .ids(&users_ids)
            .query(),
    )?
    .try_into()?;

println!("{:?}", users);
// [User { db_id: Some(DbId(2)), username: "Alice" },
//  User { db_id: Some(DbId(3)), username: "Bob" },
//  User { db_id: Some(DbId(4)), username: "John" }]
```

You can also search through the graph to get back only certain elements based on conditions. For example:

```Rust
let user: User = db
    .exec(
        &QueryBuilder::select()
            .values(User::db_keys())
            .ids(
                QueryBuilder::search()
                    .from("users")
                    .where_()
                    .key("name")
                    .value(Equal("Bob".into()))
                    .query(),
            )
            .query(),
    )?
    .try_into()?;

println!("{:?}", user);
// User { db_id: Some(DbId(3)), username: "Bob" }
```

For database concepts and primitive data types see [concepts](docs/concepts.md). For comprehensive overview of all queries see the [queries](docs/queries.md) reference or continue with more in-depth [efficient agdb](docs/efficient_agdb.md).

# Roadmap

The following are planned features in priority order:

| Feature                           | Description                                                                                                                                                                                              |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ability to disable memory mapping | Memory mapping aids with read performance but for databases larger than few GBs it is not very practical. To allow larger databases disable memory mapping and do reads from the database file directly. |
| Object query (de)serialization    | To facilitate use of the database from other languages or process the query objects and results must allow (de)serialization.                                                                            |
| Server mode                       | Executable version of the database to be accessed via network (REST & websocket).                                                                                                                        |
| Data replication & RAFT protocol  | Allow replication by connecting several database nodes together with a RAFT protocol.                                                                                                                    |
| Data sharding                     | Allow sharding single database data set across multiple nodes to allow super large databases.                                                                                                            |

# Reference

## [Efficient agdb](docs/efficient_agdb.md)

## [Concepts](docs/concepts.md)

## [Queries](docs/queries.md)

## [But why?](docs/but_why.md)
