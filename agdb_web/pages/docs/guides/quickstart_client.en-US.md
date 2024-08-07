---
title: "Quickstart - client"
description: "Quickstart - client, Agnesoft Graph Database."
---

# Quickstart

The following is the quickstart guide for the agdb client in Rust (connecting to the server). It assumes an `agdb_server` is running locally. Please refer to the [server guide](docs/guides/server) to learn how to run the server.

Looking for... [how to run a server?](docs/guides/how_to_run_server) | [another language?](/api) | [embedded/application guide?](/docs/guides/quickstart)

<br/>1. First install Rust toolchain from the [official source](https://www.rust-lang.org/tools/install) (mininum required version is `1.75.0`).
<br/>

<br/>2. Create an applicaiton folder, for example `agdb_client` and initialize your application using cargo:
<br/><br/>

```
cargo add agdb_client
```

<br/>3. Add `agdb`, `agdb_api`, `tokio` and `anyhow` as a dependencies:
<br/><br/>

```bash
cargo add agdb --features serde,openapi
cargo add agdb_api --features reqwest
cargo add tokio --features full
cargo add anyhow
```

<br/> 4. Create the client pointing to an `agdb` server:
<br/><br/>

```rs
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;

#[tokio:main]
async fn main() -> anyhow::Result<()> {
    let mut client = AgdbApi::new(ReqwestClient::new(), "localhost:3000");

    Ok(())
}
```

<br/> 5. Login as admin and setup the user:
<br/><br/>

```rs
client.user_login("admin", "admin").await?; // The authentication login is stored in
                                            // the client for subsequent API calls.
                                            // Default admin credentials are "admin/admin".
client.admin_user_add("my_user", "password123").await?;
client.user_login("my_user", "password123").await?; // Login as our newly created user.
```

<br/> 6. Create a database:
<br/><br/>

```rs
use agdb_api::DbType;

client.db_add("my_user", "my_db", DbType::Mapped).await?; // Memory mapped database called "my_db"
                                                          // will be created under our "my_user".
```

<br/> 7. Run our first queries against the database inserting a node with alias "users" and some users connecting them together:
<br/><br/>

```rs
// We derive from agdb::UserValue
// so we can use the type in the db.
#[derive(Debug, UserValue)]
struct User {
    db_id: Option<DbId>, // The db_id member is optional but
                         // it allows querying your user type
                         // from the database.
    username: String
    age: u64,
}

let users = vec![User { db_id: None, username: "Alice".to_string(), age: 40 },
                 User { db_id: None, username: "Bob".to_string(), age: 30 },
                 User { db_id: None, username: "John".to_string(), age: 20 }];

// We can pass users directly as
// query paramter thanks to the
// implementation of the agdb::DbUserValue
// trait via #[derive(UserValue)].
let queries: Vec<QueryType> = vec![QueryBuilder::insert().nodes().aliases("users").query().into(),
                                   QueryBuilder::insert().nodes().values(&users).query().into(),
                                   QueryBuilder::insert().edges().from("users").to(":1").query().into(),
                                   // We can feed the result of a previous query directly to the next one
                                   // referencing with as index into the (previous) results starting
                                   // with semicolon followed by the index, e.g. :1.
                            ];

client.exec("my_user", "my_db", &queries).await?;
```

<br/> 8. Run another query searching & selecting the users and converting them back to the native local object:
<br/><br/>

```rs
let queries = vec![QueryBuilder::select()
                    .values(User::db_keys()) // Select only relevant properties for the User struct.
                    .ids(
                        QueryBuilder::search()
                            .from("users") // Start the search from the "users" node.
                            .where_()
                            .key("age") // Examine "age" property.
                            .value(LessThan(40.into())) // Include it in the search result if the value
                                                        // is less than 40.
                            .query(),
                    )
                    .query()];

// Runs the query against the db, grabs the first result and converts it to the collection of users.
let users: Vec<User> = client.exec("my_user", "my_db", &queries).await?[0].try_into()?;

println!("{:?}", users);
```

<br/> 9. Full program:
<br/><br/>

Cargo.toml:

```
[package]
name = "agdb_client"
edition = "2021"

[dependencies]
anyhow = "1"
agdb = { version = "0.5.2", path = "../agdb", features = ["serde", "openapi"] }
agdb_api = { version = "0.5.2", path = "../agdb_api/rust", features = ["reqwest"] }
tokio = { version = "1", features = ["full"] }
```

main.rs

```rs
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use agdb_api::DbType;

#[derive(Debug, UserValue)]
struct User {
    db_id: Option<DbId>,
    username: String
    age: u64,
}

#[tokio:main]
async fn main() -> anyhow::Result<()> {
    let mut client = AgdbApi::new(ReqwestClient::new(), "localhost:3000");

    client.user_login("admin", "admin").await?;
    client.admin_user_add("my_user", "password123").await?;
    client.user_login("my_user", "password123").await?;


    client.db_add("my_user", "my_db", DbType::Mapped).await?;

    let users = vec![User { db_id: None, username: "Alice".to_string(), age: 40 },
                    User { db_id: None, username: "Bob".to_string(), age: 30 },
                    User { db_id: None, username: "John".to_string(), age: 20 }];

    let queries: Vec<QueryType> = vec![QueryBuilder::insert().nodes().aliases("users").query().into(),
                                       QueryBuilder::insert().nodes().values(&users).query().into(),
                                       QueryBuilder::insert().edges().from("users").to(":1").query().into()]

    client.exec("my_user", "my_db", &queries).await?;

    let queries = vec![QueryBuilder::select()
                        .values(User::db_keys())
                        .ids(
                            QueryBuilder::search()
                                .from("users")
                                .where_()
                                .key("age")
                                .value(LessThan(40.into()))
                                .query(),
                        )
                        .query()];

    let users: Vec<User> = client.exec("my_user", "my_db", &queries).await?[0].try_into()?;

    println!("{:?}", users);

    Ok(())
}
```

<br/> 10. As an excercise, can you modify the program so that it runs all queries in a single batch?
<br/><br/>

Hint:

```rs
let queries: Vec<QueryType> = vec![QueryBuilder::insert().nodes().aliases("users").query().into(),
                                   QueryBuilder::insert().nodes().values(&users).query().into(),
                                   QueryBuilder::insert().edges().from("users").to(":1").query().into(),
                                   QueryBuilder::select().values(User::db_keys()).ids(
                                        QueryBuilder::search().from("users").where_().key("age").value(LessThan(40.into())).query(),
                                   ).query()]
//...
let users: Vec<User> = client.exec("my_user", "my_db", &queries).await?[3].try_into()?; // Have you noticed a different index of the result?
```
