# Agnesoft Graph Database

![Crates.io](https://img.shields.io/crates/v/agdb) [![release](https://github.com/agnesoft/agdb/actions/workflows/release.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/release.yaml) [![coverage](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml) [![codecov](https://codecov.io/gh/agnesoft/agdb/branch/main/graph/badge.svg?token=Z6YO8C3XGU)](https://codecov.io/gh/agnesoft/agdb)

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

https://github.com/agnesoft/agdb/blob/main/tests/db_test.rs#L242-L271

# Reference

TBD
