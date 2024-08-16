---
title: "Quickstart"
description: "Quickstart, Agensoft Graph Database"
---

# Quickstart

The following is the quickstart guide for the agdb emebedded/application database.

[Looking for server client guide instead?](/docs/guides/quickstart-client)

1. First install Rust toolchain from the [official source](https://www.rust-lang.org/tools/install) (mininum required version is `1.75.0`).

2. Create an applicaiton folder, for example `agdb_app` and initialize your application using cargo:

```bash
mkdir agdb_app
cd agdb_app
cargo init
```

3. Add `agdb` as a dependency:

```bash
cargo add agdb
```

4. Create the memory mapped database in your code:

```rs
use agdb::QueryError;

fn main() -> Result<(), QueryError> {
    let mut db = Db::new("agdb_app.agdb")?; // The namesake file "agdb_app.agdb" will
                                            // be created in your working directory.
                                            // The .agdb extension is conventional.
    Ok(())
}
```

5. Run your first query against the database inserting a node with alias "users":

```rs
db.exec_mut(&QueryBuilder::insert()
                .nodes()
                .aliases("users")
                .query())?;
```

6. Insert additional nodes representing some users and connect them with the "users" node:
   <br><br>

```rs
// We derive from agdb::UserValue
// so we can use the type in the db.
#[derive(Debug, UserValue)]
struct User {
    db_id: Option<DbId>, // The db_id member is optional but
                         // it allows insert your user type
                         // directly into the database.
    username: String,
    age: u64,
}

let users = vec![User { db_id: None, username: "Alice".to_string(), age: 40 },
                 User { db_id: None, username: "Bob".to_string(), age: 30 },
                 User { db_id: None, username: "John".to_string(), age: 20 }];

let db_users = db.exec_mut(&QueryBuilder::insert()
                                .nodes()
                                .values(&users) // We can pass users directly as
                                                // query paramter thanks to the
                                                // implementation of the agdb::DbUserValue
                                                // trait via #[derive(UserValue)].
                                .query())?;

db.exec_mut(
    &QueryBuilder::insert()
        .edges()
        .from("users")
        .to(&db_users) // We can feed result of a previous
                       // query directly to the next one.
        .query(),
)?;
```

7. Find a user in the database matching some conditions:

```rust
// We combine search & select into a single query like so:
let users: Vec<User> = db
    .exec(
        &QueryBuilder::select()
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
            .query(),
    )?
    .try_into()?; // Convert the result into a list of User objects.

println!("{:?}", users);
// We can print the users thanks to #[derive(Debug)]. The result should be something like:
// Vec [User { db_id: Some(DbId(3)), username: "John", age: 20 }, User { db_id: Some(DbId(3)), username: "Bob", age: 30 }]
```

8. Full program: https://github.com/agnesoft/agdb/tree/main/examples/app_db
