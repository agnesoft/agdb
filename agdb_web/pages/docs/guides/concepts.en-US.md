---
title: "Concepts"
description: "Concepts, Agnesoft Graph Database"
---

# Concepts

## Graph

_Related:_ [Why graph?](/blog/why-graph)

Graph is a set of nodes (also vertices, points) that are connected to each other through edges (also arcs, links). In `agdb` the data is plotted on directed graphs and there are no restrictions on their structure. They can be cyclic (forming a cycle), acyclic (being open ended), sparse (having only some connections between nodes), disjointed (thus forming multiple graphs), having self-referential edges (nodes being connected to themselves), having multiple edges to the same node (even itself) and/or in the same same direction.

Nodes and edges are `graph elements` and each can have key-value pairs associated with them usually referred to as `values`. Each graph element has a signed integer id (db id) - nodes having positive values while edges negative values. Note that upon removal of a graph element its id is freed and can be reused by subsequent inserts of new graph elements.

**Terminology:**

-   Graph (set of nodes and edges)
-   Node (point on a graph)
-   Edge (connection between two nodes)
-   Graph elements (nodes & edges)
-   db id (graph element id, positive for nodes, negative for edges)
-   Values (key-value pairs associated with a node or an edge)

## Query

_Related:_ [Why object queries?](/blog/object-queries), [Queries](/docs/references/queries)

Query is a request to retrieve or manipulate data in a database (both the graph structure and `values` associated with the nodes and edges). In `agdb` queries are not texts (like in SQL) but rather objects that contain details about what is being requested. These objects are typically constructed via a query builder but it is also possible to create them like any other object. The builder steps resemble, and often indeed are, direct translations of a well known SQL equivalents (e.g. `QueryBuilder::select() == SELECT`, `QueryBuilder::insert() == INSERT INTO`).

Queries are executed by the database directly. The `agdb` distinguishes between `immutable` (retrieving data) and `mutable` (manipulating data) queries. Each query execution produces either a result or an error. In `agdb` there is a single `result` object containing a numerical result (i.e. number of affected elements or values) and a list of database elements. Each element in a result is comprised of a database id and a list of `values` (associated key-value pairs).

In case of a failure the database execution yields an error detailing what went wrong instead of a result.

See dedicated [queries](/docs/references/queries) documentation for details.

**Terminology:**

-   Query (request to retrieve or manipulate data)
-   Immutable query (request to retrieve data)
-   Mutable query (request to manipulate data)
-   Result (result of a query)

## Transaction

_Related_: [Queries](/docs/references/queries)

Transactions are a way to provide atomicity, isolation and data consistency in a database (three of [ACID](https://en.wikipedia.org/wiki/ACID) properties). In `agdb` every query is a transaction but it is also possible to execute multiple queries as a single transaction. Just like `queries` transactions are immutable or mutable. One important rule is borrowed directly from Rust and enforced on the type level:

_"There can be either unlimited number of concurrent immutable transactions or exactly one mutable transaction"_

In multithreaded environment you can easily synchronize the access to the database by using [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html). Furthermore unlike traditional transactions implemented in other database systems the `agdb` transactions are immediately executed requiring a closure containing (minimum) amount of code and queries required for the transaction to be performed. This forces the client to optimize their transactions and reduce the time the database is locked, which is particularly important for mutable transactions as they lock the entire database for their execution.

**Terminology:**

-   Transaction (set of queries to be executed atomically against a database wrapped in a closure)
-   Mutable transaction (set of mutable & immutable queries wrapped in a closure)
-   Immutable transaction (set of immutable queries wrapped in a closure)

## Storage

_Related_: [Why single file?](/blog/single-file)

Every persistent database eventually stores its data somewhere on disk in one or more files. the `agdb` stores its data in a single file (that is being shadowed by another temporary write ahead log file). Its internal structure is very similar to that of a memory which makes it very easy to map between the two. The file format is fully platform agnostic and the file can be safely transferred to another machine and loaded there. Similarly the `agdb` is by default memory mapped database but it could just as easily operate purely on the file itself at the cost of read performance (might be implemented as a feature in the future).

The database durability is provided by the write ahead log (WAL) file which records reverse of every operation to be performed on the main file before it actually happens. In case of any catastrophic failure the main database file is repaired from the WAL on loading the database.

Just like the memory the main database file will get fragmented over time. Sectors of the file used for the data that was later reallocated will remain unused (fragmented) until the database file is defragmented. That operation is performed automatically on database object instance drop.

The storage taken by individual elements are properties is generally as follows:

-   node: 32 bytes
-   edge: 32 bytes
-   single key or value (<=15 bytes): 16 bytes
-   single key or value (>15 bytes): 32 bytes (+)
-   key-value pair: 32 bytes (+)

The size of the graph elements (nodes & edges) is fixed. The size of the properties (key-value pairs) is at least 32 bytes (16 per key and 16 per value) but can be greater if the value itself is greater. This creates some inefficiency for small values (e.g. integers) but it also allows application of small value optimization where values up to 15 bytes in size (e.g. strings) do not allocate or take extra space. When a value is larger than 15 bytes it will be stored separately with another 16 bytes overhead making it at least `32 + value length` bytes.

The reason for values taking 16 bytes at minimum instead of 8 is that the value needs to store a type information for which 1 byte is required. 9 bytes is an awkward and very inefficient (as measured where 16 byte values were much faster) size even if it could save some file space. The next alignment is therefore 16 bytes which also allows the aforementioned small value optimization.

**Terminology:**

-   File storage (underlying single data file)
-   Write ahead log (WAL, shadowing file storage to provide durability)

## Data types

Supported types of both keys and values are:

-   `i64`
-   `u64`
-   `f64`
-   `String`
-   `Vec<u8>`
-   `Vec<i64>`
-   `Vec<u64>`
-   `Vec<f64>`
-   `Vec<String>`

It is an enum of limited number of supported types that are universal across all platforms and programming languages. They are serialized in file as follows:

| Type          | Layout                                                                          | Size     |
| ------------- | ------------------------------------------------------------------------------- | -------- |
| `i64`         | little endian                                                                   | 8 bytes  |
| `u64`         | little endian                                                                   | 8 bytes  |
| `f64`         | little endian                                                                   | 8 bytes  |
| `String`      | size as `u64` little endian followed by UTF-8 encoded string as `u8` bytes      | 8+ bytes |
| `Vec<u8>`     | size as `u64` little endian followed by individual `u8` bytes                   | 8+ bytes |
| `Vec<i64>`    | size as `u64` little endian followed by individual `i64` little endian elements | 8+ bytes |
| `Vec<u64>`    | size as `u64` little endian followed by individual `u64` little endian elements | 8+ bytes |
| `Vec<f64>`    | size as `u64` little endian followed by individual `f64` elements               | 8+ bytes |
| `Vec<String>` | size as `u64` little endian followed by individual `String` elements            | 8+ bytes |
