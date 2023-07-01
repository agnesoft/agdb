# Agnesoft Graph Database

[![Crates.io](https://img.shields.io/crates/v/agdb)](https://crates.io/crates/agdb) [![release](https://github.com/agnesoft/agdb/actions/workflows/release.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/release.yaml) [![coverage](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml) [![codecov](https://codecov.io/gh/agnesoft/agdb/branch/main/graph/badge.svg?token=Z6YO8C3XGU)](https://codecov.io/gh/agnesoft/agdb)

The Agnesoft Graph Database (aka _agdb_) is persistent memory mapped graph database using object 'no-text' queries. It can be used as a main persistent storage, data analytics platform as well as fast in-memory cache. Its typed schema-less data store allows for flexible and seamless data updates with no downtime or costly migrations. All queries are constructed via a builder pattern (or directly as objects) with no special language or text parsing.

# Key Features

- Data plotted on a graph
- Typed key-value properties of graph elements (nodes & edges)
- Persistent file based storage
- ACID compliant
- Object queries with builder pattern (no text, no query language)
- Memory mapped for fast querying
- _No dependencies_

# Quickstart

```
cargo add agdb
```

Basic usage demonstrating creating a database, inserting graph elements with data and querying them back with select and search. The function using this code must handle `agdb::DbError` and [`agdb::QueryError`](docs/queries.md#queryerror) error types for operator `?` to work:

```Rust
let mut db = Db::new("user_db.agdb")?;

db.exec_mut(&QueryBuilder::insert().nodes().aliases("users").query())?;
let users = db.exec_mut(&QueryBuilder::insert()
                            .nodes()
                            .values(vec![vec![("username", "Alice").into(), ("joined", 2023).into()],
                                         vec![("username", "Bob").into(), ("joined", 2015).into()],
                                         vec![("username", "John").into()]])
                            .query())?;
db.exec_mut(&QueryBuilder::insert().edges().from("users").to(users.ids()).query())?;
```

This code creates a database called `user_db.agdb` with a simple graph of 4 nodes. The first node is aliased `users` and 3 user nodes for Alice, Bob and John are then connected with edges to the `users` node. The arbitrary `username` and sometimes `joined` properties are attached to the user nodes.

You can select the graph elements (both nodes & edges) with their ids to get them back with their associated data (key-value properties):

```Rust
let user_elements = db.exec(&QueryBuilder::select().ids(users.ids()).query())?;
println!("{:?}", user_elements);
// QueryResult {
//   result: 3,
//   elements: [
//     DbElement { id: DbId(2), values: [DbKeyValue { key: String("username"), value: String("Alice") }, DbKeyValue { key: String("joined"), value: Int(2023) }] },
//     DbElement { id: DbId(3), values: [DbKeyValue { key: String("username"), value: String("Bob") }, DbKeyValue { key: String("joined"), value: Int(2015) }] },
//     DbElement { id: DbId(4), values: [DbKeyValue { key: String("username"), value: String("John") }] }
// ] }
```

You can also search through the graph to get back only the elements you want:

```Rust
let user = db.exec(&QueryBuilder::select()
                        .search(QueryBuilder::search()
                                    .from("users")
                                    .where_()
                                    .key("username")
                                    .value(Comparison::Equal("John".into()))
                                    .query())
                        .query())?;
println!("{:?}", user);
// QueryResult {
//   result: 1,
//   elements: [
//     DbElement { id: DbId(4), values: [DbKeyValue { key: String("username"), value: String("John") }] }
//   ] }
```

For comprehensive overview of all queries see the [queries](docs/queries.md) reference or continue with more in-depth [guide](docs/guide.md).

# Reference

- [Concepts](docs/concepts.md)
- [Queries](docs/queries.md)
- [Guide](docs/guide.md)
- [But why?](docs/but_why.md)
