<p align="center">
    <a href="https://agdb.agnesoft.com/"><img width="300" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo"></a>
</p>
<h1 align="center">
    agdb
</h1>
<h3 align="center">
    The graph database.
</h3>

<p align="center">
    <a href="https://agdb.agnesoft.com/docs/references/queries"><img width="100" src="https://agdb.agnesoft.com/images/db.png" alt="db"></a> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    <a href="https://agdb.agnesoft.com/api-docs/openapi"><img width="100" src="https://agdb.agnesoft.com/images/api.png" alt="api"></a> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    <a href="https://agdb.agnesoft.com/docs/references/studio"><img width="100" src="https://agdb.agnesoft.com/images/studio.png" alt="studio"></a> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    <a href="https://agdb.agnesoft.com/docs/references/server"><img width="100" src="https://agdb.agnesoft.com/images/server.png" alt="server"></a> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    <a href="https://agdb.agnesoft.com/enterprise/cloud"><img width="100" src="https://agdb.agnesoft.com/images/cloud.png" alt="cloud"></a> 
</p>

<p align="center">
    <a href="https://agdb.agnesoft.com/api-docs/rust"><img width="25" src="https://agdb.agnesoft.com/images/rust.png" alt="rust"></a>
    <a href="https://agdb.agnesoft.com/api-docs/typescript"><img width="25" src="https://agdb.agnesoft.com/images/ts.png" alt="ts"></a>
    <a href="https://agdb.agnesoft.com/api-docs/typescript"><img width="25" src="https://agdb.agnesoft.com/images/js.png" alt="js"></a>
    <a href="https://agdb.agnesoft.com/api-docs/php"><img width="45" src="https://agdb.agnesoft.com/images/php.svg" alt="js"></a>
    <img width="25" src="https://agdb.agnesoft.com/images/python.png" alt="python">
    <img width="25" src="https://agdb.agnesoft.com/images/java.png" alt="java">
    <img width="25" src="https://agdb.agnesoft.com/images/c.png" alt="c">
    <img width="25" src="https://agdb.agnesoft.com/images/cpp.png" alt="cpp">
    <img width="25" src="https://agdb.agnesoft.com/images/csharp.png" alt="csharp">
</p>

<p align="center">
    <a href="./LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="license"></a>
    <a href="https://crates.io/crates/agdb"><img src="https://img.shields.io/crates/v/agdb" alt="Crates.io"></a>
    <a href="https://docs.rs/agdb/latest/agdb/"><img src="https://docs.rs/agdb/badge.svg"></a>
    <a href="https://github.com/agnesoft/agdb/actions/workflows/release.yaml"><img src="https://github.com/agnesoft/agdb/actions/workflows/release.yaml/badge.svg" alt="release"></a>
    <a href="https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml"><img src="https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml/badge.svg" alt="coverage"></a>
    <a href="https://codecov.io/gh/agnesoft/agdb"><img src="https://codecov.io/gh/agnesoft/agdb/branch/main/graph/badge.svg?token=Z6YO8C3XGU" alt="codecov"></a>
</p>

<!-- <p align="center">
    <img width="25" src="https://agdb.agnesoft.com/images/reddit.png" alt="reddit"> &nbsp;
    <img width="25" src="https://agdb.agnesoft.com/images/x.png" alt="x"> &nbsp;
    <img width="25" src="https://agdb.agnesoft.com/images/linkedin.png" alt="lkinkedin"> &nbsp;
    <img width="25" src="https://agdb.agnesoft.com/images/stackoverflow.png" alt="stackoverflow"> &nbsp;
</p> -->

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;Agnesoft Graph Database

<p align="center">
    <a href="https://agdb.agnesoft.com/docs/guides/quickstart">Quickstart Db</a> | <a href="https://agdb.agnesoft.com/api-docs/rust">Quickstart Client</a> | <a href="https://agdb.agnesoft.com/docs/references/queries">QUERIES</a> | <a href="#decision-tree">DECISION TREE</a>
</p>
<p align="center">
    <a href="https://agdb.agnesoft.com/blog/blog">Why not SQL?</a>
</p>

The Agnesoft Graph Database (aka _agdb_) is application native database without compromises. It is persistent, optionally memory mapped graph database with native object 'no-text' queries. It can be used as a main persistent storage, data analytics platform as well as fast in-memory cache. Its typed schema-less data store allows for flexible and seamless data updates with no downtime or costly migrations. All queries are constructed via a builder pattern or directly as objects with no special language or text parsing.

- [Key Features](#key-features)
- [At a glance](#at-a-glance-db)
- [Crate Features](#crate-features)
- [Decision Tree](#decision-tree)
- [Roadmap](#roadmap)
- [Reference](#reference)

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;Key Features

- The `agdb` is application native database.
- No query language, queries are written in the same language as the application.
- Performance without limits, constant time lookups and traversals regardless of db size.
- Simple to build, use & scale.

Technical features:

- Data plotted on a graph
- Typed [key-value properties](https://agdb.agnesoft.com/docs/guides/concepts) attached to graph elements (nodes & edges)
- Persistent platform agnostic file based storage (transferable between platforms)
- [ACID](https://en.wikipedia.org/wiki/ACID) compliant
- [Object queries](https://agdb.agnesoft.com/docs/references/queries) with builder pattern (no text, no query language)
- Memory mapped for fast querying
- [Server mode](https://agdb.agnesoft.com/docs/references/server)
- [Cluster mode](https://agdb.agnesoft.com/docs/references/server#cluster)
- In-built TLS support
- [OpenAPI clients](https://agdb.agnesoft.com/api-docs/openapi) in any programming language
- [Cloud](https://agdb.agnesoft.com/enterprise/cloud) hosted SaaS database
- _Db itself has no dependencies_

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;At a glance [Db]

```
cargo add agdb
```

Basic usage demonstrating creating a database, inserting graph elements with data and querying them back with select and search. The function using this code must handle [`agdb::DbError`](https://agdb.agnesoft.com/docs/references/queries#dberror) for operator `?` to work (i.e. `fn foo() -> Result<(), agdb::DbError>`):

```rs
use agdb::{Db, DbId, QueryBuilder, DbType, Comparison::Equal};

let mut db = Db::new("db_file.agdb")?;

db.exec_mut(QueryBuilder::insert().nodes().aliases("users").query())?;

#[derive(Debug, DbType)]
struct User { db_id: Option<DbId>, name: String, }
let users = vec![User { db_id: None, name: "Alice".into(), },
                 User { db_id: None, name: "Bob".into(), },
                 User { db_id: None, name: "John".into(), }];

let users_ids = db.exec_mut(QueryBuilder::insert().nodes().values(&users).query())?;

db.exec_mut(
    QueryBuilder::insert()
        .edges()
        .from("users")
        .to(&users_ids)
        .query(),
)?;
```

This code creates a database called `user_db.agdb` with a simple graph of 4 nodes. The first node is aliased `users` and 3 user nodes for Alice, Bob and John are then connected with edges to the `users` node. The arbitrary `name` property is attached to the user nodes. Rather than inserting values directly with keys (which is also possible) we use our own type and derive from `agdb::DbType` to allow it to be used with the database.

You can select the graph elements (both nodes & edges) with their ids to get them back with their associated data (key-value properties). Let's select our users and convert the result into the list (notice we select only values relevant to our `User` type with passing `User::db_keys()`):

```rs
let users: Vec<User> = db
    .exec(
        QueryBuilder::select()
            .elements::<User>()
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

```rs
let user: User = db
    .exec(
        QueryBuilder::select()
            .elements::<User>()
            .search()
            .from("users")
            .where_()
            .key("name")
            .value(Equal("Bob".into()))
            .query(),
    )?
    .try_into()?;

println!("{:?}", user);
// User { db_id: Some(DbId(3)), username: "Bob" }
```

For database concepts and primitive data types see [concepts](https://agdb.agnesoft.com/docs/guides/concepts). For comprehensive overview of all queries see the [queries](https://agdb.agnesoft.com/docs/references/queries) reference or continue with more in-depth [efficient agdb](https://agdb.agnesoft.com/docs/references/efficient-agdb).

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;Crate Features

### agdb

| Feature  | Default | Description                                                                                                         |
| -------- | ------- | ------------------------------------------------------------------------------------------------------------------- |
| derive   | yes     | Enables derive macro to enable custom user types to be directly used with the database.                             |
| opeanapi | no      | Enables `ToSchema` macro on query structs so they can be exported to json OpeanAPI/Swagger schema.                  |
| serde    | no      | Enables serialiation/deserialization of queries and QueryResult using [`serde`](https://github.com/serde-rs/serde). |
| api      | no      | Enables annotations on all structs to facilitate API generation for different languages.                            |

### agdb_api

| Feature    | Default | Description                                                                              |
| ---------- | ------- | ---------------------------------------------------------------------------------------- |
| rust-tls   | no      | Enables rust-tls for [`reqwest`](https://github.com/seanmonstar/reqwest).                |
| native-tls | no      | Enables native-tls for [`reqwest`](https://github.com/seanmonstar/reqwest).              |
| api        | no      | Enables annotations on all structs to facilitate API generation for different languages. |

### agdb_server

| Feature | Default | Description                                                                    |
| ------- | ------- | ------------------------------------------------------------------------------ |
| tls     | no      | Enables TLS support via `rustls`. On Windows requires MSVC and CMake to build. |
| studio  | no      | Embedd the `agdb_studio` into the server at `/studio` route.                   |

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;Decision Tree

```mermaid
flowchart TD;
    A[Embedded or server?] --> Embedded
    A --> B[Client or hosting?]
    Embedded --> Studio[<a href='https://agdb.agnesoft.com/docs/references/studio'>Studio</a>]
    Embedded --> Queries[<a href='https://agdb.agnesoft.com/docs/references/queries'>Queries</a>]
    B --> Client
    B --> Hosting
    Client --> API[<a href='https://agdb.agnesoft.com/api-docs/openapi'>API</a>]
    Client --> Studio
    Client --> Queries
    Hosting --> Server[<a href='https://agdb.agnesoft.com/docs/references/server'>Server</a>]
    Hosting --> Cloud[<a href='https://agdb.agnesoft.com/enterprise/cloud'>Cloud</a>]
```

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;Roadmap

The following are planned features:

| Feature               | Description                                                                                                                                          |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Agdb Studio           | Graphical interface to `agdb`                                                                                                                        |
| Python Client         | Convenience client using bindings genereated from OpenAPI.                                                                                           |
| Java Client           | Convenience client using bindings genereated from OpenAPI.                                                                                           |
| C# Client             | Convenience client using bindings genereated from OpenAPI.                                                                                           |
| C Client              | Convenience client using bindings genereated from OpenAPI.                                                                                           |
| C++ Client            | Convenience client using bindings genereated from OpenAPI.                                                                                           |
| Agdb Playground       | Free public cloud-based playground to tinker with `agdb`.                                                                                            |
| #\[no_std]            | The `agdb` does not require any dependencies and thus should be (in theory) `no_std` friendly but it will likely require some development & testing. |
| Public Cloud Offering | Commercial & supported `agdb` instance hosted in a public cloud.                                                                                     |

## <img width="25" src="https://agdb.agnesoft.com/images/logo.svg" alt="agdb logo">&nbsp;&nbsp;Reference

- [API](https://agdb.agnesoft.com/api-docs/openapi)

- [But why?](https://agdb.agnesoft.com/blog/blog)

- [Cloud](https://agdb.agnesoft.com/enterprise/cloud)

- [Concepts](https://agdb.agnesoft.com/docs/guides/concepts)

- [Efficient agdb](https://agdb.agnesoft.com/docs/references/efficient-agdb)

- [Guides](https://agdb.agnesoft.com/docs/guides)

- [Performance](https://agdb.agnesoft.com/docs/references/performance)

- [Queries](https://agdb.agnesoft.com/docs/references/queries)

- [Server](https://agdb.agnesoft.com/docs/references/server)

- [Studio](https://agdb.agnesoft.com/docs/references/studio)

- [Troubleshooting](https://agdb.agnesoft.com/docs/guides/troubleshooting)
