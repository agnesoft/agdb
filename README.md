# Agnesoft Graph Database

![Crates.io](https://img.shields.io/crates/v/agdb) [![release](https://github.com/agnesoft/agdb/actions/workflows/release.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/release.yaml) [![coverage](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml/badge.svg)](https://github.com/agnesoft/agdb/actions/workflows/coverage.yaml) [![codecov](https://codecov.io/gh/agnesoft/agdb/branch/main/graph/badge.svg?token=Z6YO8C3XGU)](https://codecov.io/gh/agnesoft/agdb)

The Agnesoft Graph Database (aka _agdb_) is persistent memory mapped graph database using object 'no-text' queries. It can be used as a main persistent storage, data analytics platform as well as fast in-memory cache. Its typed schema-less data store allows for flexible and seamless data updates with no downtime or costly migrations. All queries are constructed via a builder pattern (or directly as objects) with no special language or text parsing.

# Key Features

- Persistent file based storage
- Memory mapped for fast querying
- ACID compliant
- Object queries (no text, no query language)
- Typed schema-less key-value data store
- _No dependencies_

# Quickstart

```
cargo add agdb
```

Basic usage demonstrating creating a database, inserting graph elements with data and querying them back with select and search:

https://github.com/agnesoft/agdb/blob/c1f684916a0f45fd086de7824ad213e770613470/tests/examples.rs#L13-L48

# Reference

- [Concepts](docs/concepts.md)
- [Queries](docs/queries.md)
- [Implementation](docs/implementation.md)
- [But why?](docs/but_why.md)
