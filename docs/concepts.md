# Concepts

## Graph

_Related:_ [Why graph?](but_why.md#why-graph)

Graph is a set of nodes (also vertices, points) that are connected to each other through edges (also arcs, links). In `agdb` the data is plotted on directed graphs and there are no restrictions on their structure. They can be cyclic (forming a cycle), acyclic (being open ended), sparse (having only some connections between nodes), disjointed (thus forming multiple graphs), having self-referential edges (nodes being connected to themselves), having multiple edges to the same node (even itself) and/or in the same same direction.

Nodes and edges are `graph elements` and each can have key-value pairs associated with them usually referred to as `values`. Each graph element has a signed integer id (db id) - nodes having positive values while edges negative values. Note that upon removal of a graph element its id is freed and can be reused by subsequent inserts of new graph elements.

Terminology:

- Graph (set of nodes and edges)
- Node (point on a graph)
- Edge (connection between two nodes)
- Graph elements (nodes & edges)
- db id (graph element id, positive for nodes, negative for edges)
- Values (key-value pairs associated with a node or an edge)

## Query

_Related:_ [Why object queries?](but_why#why-object-queries)

Query is a request to retrieve or manipulate data in a database (both the graph structure and `values` associated with the nodes and edges). In `agdb` queries are not texts (like in SQL) but rather objects that contain details about what is being requested. These objects are typically constructed via a query builder but it is also possible to create them like any other object. The builder steps resemble, and often indeed are, direct translations of a well known SQL equivalents (e.g. `QueryBuilder::select() == SELECT`, `QueryBuilder::insert() == INSERT INTO`).

Queries are executed by the database directly. The `agdb` distinguishes between `immutable` (retrieving data) and `mutable` (manipulating data) queries. Each query execution produces either a result or an error. In `agdb` there is a single `result` object containing a numerical result (i.e. number of affected elements or values) and a list of database elements. Each element in a result is comprised of a database id and a list of `values` (associated key-value pairs).

In case of a failure the database execution yields an error detailing what went wrong instead of a result.

See dedicated [queries](queries.md) documentation for details.

Terminology:

- Query (request to retrieve or manipulate data)
- Immutable query (request to retrieve data)
- Mutable query (request to manipulate data)
- Result (result of a query)

## Transaction

Transactions are a way to provide atomicity, isolation and data consistency in a database (three of [ACID](https://en.wikipedia.org/wiki/ACID) properties). In `agdb` every query is a transaction but it is also possible to execute multiple queries as a single transaction. Just like `queries` transactions are immutable or mutable. One important rule is borrowed directly from Rust and enforced on the type level:

_"There can be either unlimited number of concurrent immutable transactions or exactly one mutable transaction"_

In multithreaded environment you can easily synchronize the access to the database by using [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html). Furthermore unlike traditional transactions implemented in other database systems the `agdb` transactions are immediately executed requiring a closure containing (minimum) amount of code and queries required for the transaction to be performed. This forces the client to optimize their transactions and reduce the time the database is locked, which is particularly important for mutable transactions as they lock the entire database for their execution.

See dedicated [queries](queries.md) documentation for details.

Terminology:

- Transaction (set of queries to be executed atomically against a database wrapped in a closure)
- Mutable transaction (set of mutable & immutable queries wrapped in a closure)
- Immutable transaction (set of immutable queries wrapped in a closure)

## Storage

Every persistent database eventually stores its data somewhere on disk in one or more files. the `agdb` stores its data in a single file (that is being shadowed by another temporary write ahead log file). Its internal structure is very similar to that of a memory which makes it very easy to map between the two. The file format is fully platform agnostic and the file can be safely transferred to another machine and loaded there. Similarly the `agdb` is by default memory mapped database it could just as easily operate purely on the file itself at the cost of read performance.

TBD (durability, defragmentation)

Terminology:

- File storage (underlying single data file)
- Write ahead log (shadowing file storage to provide durability)
