# Agnesoft Graph Database

![Crates.io](https://img.shields.io/crates/v/agdb)[![release](https://github.com/agnesoft/agdb/actions/workflows/build.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/release.yaml) [![coverage](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml) [![codecov](https://codecov.io/gh/agnesoft/agdb/branch/main/graph/badge.svg?token=Z6YO8C3XGU)](https://codecov.io/gh/agnesoft/agdb)

The Agnesoft Graph Database (aka _agdb_) is persistent memory mapped graph database using purely 'no-text' programmatic queries. It can be used as a main persistent storage as well as fast in-memory cache. Its typed but schema-less data store allows for seamless data updates with no downtime or costly migrations. All queries are constructed via a builder pattern (or directly as objects) with no special language or text parsing.

# Key Features

- Persistent file based storage
- Memory mapped for fast querying
- ACID compliant
- Programmatic queries (no text, no query language)
- Typed schema-less key-value data store

# Quickstart

Add `agdb` as a dependency to your project:

```
cargo add agdb
```

Basic usage demonstrating creating a database, insert the graph elements with data and querying them back (select and search):

```
use agdb::Db;
use agdb::Comparison;

fn main() {
    let mut db = Db::new("db_file.agdb").unwrap();

    //create a nodes with data
    db.exec_mut(&QueryBuilder::insert().nodes().aliases(&["users".into()]).query()).unwrap();
    let users = db.exec_mut(&QueryBuilder::insert().nodes().values(&[
        &[("id", 1).into(), ("username", "user_1").into()],
        &[("id", 2).into(), ("username", "user_2").into()],
        &[("id", 3).into(), ("username", "user_3").into()]]
    ).query()).unwrap();

    //connect nodes
    db.exec_mut(&QueryBuilder::insert().edges().from(&["users".into()]).to(&users.ids()).query()).unwrap();

    //select nodes
    let user_elements = db.exec(&QueryBuilder::select().ids(&users.ids()).query()).unwrap();

    for element in user_elements.elements {
        println!("{:?}: {:?}", element.id, element.values);
    }

    // DbId(2): [DbKeyValue { key: String("id"), value: Int(1) }, DbKeyValue { key: String("username"), value: String("user_1") }]
    // DbId(3): [DbKeyValue { key: String("id"), value: Int(2) }, DbKeyValue { key: String("username"), value: String("user_2") }]
    // DbId(4): [DbKeyValue { key: String("id"), value: Int(3) }, DbKeyValue { key: String("username"), value: String("user_3") }]

    //search with conditions
    let user_id = db.exec(&QueryBuilder::search().from("users").where_().key("username").value(Comparison::Equal("user_2".into())).query()).unwrap();

    println!("{:?}", user_id.elements);
    //[DbElement { id: DbId(3), values: [] }]
}
```

# Reference

TBD
