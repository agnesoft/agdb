- [QueryResult](#queryresult)
- [QueryError](#queryerror)
- [Transactions](#transactions)
- [QueryIds \& QueryId](#queryids--queryid)
- [QueryValues](#queryvalues)
- [Mutable queries](#mutable-queries)
  - [Insert](#insert)
    - [Insert nodes](#insert-nodes)
    - [Insert edges](#insert-edges)
    - [Inserted aliases](#inserted-aliases)
    - [Insert values](#insert-values)
  - [Remove](#remove)
    - [Remove elements](#remove-elements)
    - [Remove aliases](#remove-aliases)
    - [Remove values](#remove-values)
- [Immutable queries](#immutable-queries)
  - [Select](#select)
    - [Select elements](#select-elements)
    - [Select values](#select-values)
    - [Select keys](#select-keys)
    - [Select key count](#select-key-count)
    - [Select aliases](#select-aliases)
    - [Select all aliases](#select-all-aliases)
  - [Search](#search)

All interactions with the `agdb` are realized through queries. There are two kinds of queries:

- Immutable queries
- Mutable queries

Immutable queries read the data from the database through `select` and `search` queries. Mutable queries write to or delete from the database through `insert` and `remove` queries. All queries follow the Rust rules about borrowing:

```
There can be unlimited number of immutable concurrent queries or exactly one mutable query running against the database.
```

The queries are executed against the database by calling the corresponding method on the database object:

```Rust
impl Db {
    // immutable queries only
    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError>

    // mutable queries only
    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError>
}
```

Alternatively you can run a series of queries as a [transaction](#transactions).

All queries return `Result<QueryResult, QueryError>`. The [`QueryResult`](#queryresult) is the universal data structure holding results of all queries in an uniform structure. The [`QueryError`](#queryerror) is the singular error type holding information of any failure or problem encountered when running the query.

# QueryResult

The `QueryResult` is the universal result type for all successful queries. It looks like:

```Rust
pub struct QueryResult {
    pub result: i64,
    pub elements: Vec<DbElement>,
}
```

The `result` field holds numerical result of the query. It typically returns the number of database items affected. For example when selecting from the database it will hold a positive number of elements returned. When removing from the database it will hold a negative number of elements deleted from the database.

The `elements` field hold the [database elements](concepts.md#graph) returned. Each element looks like:

```Rust
pub struct DbElement {
    pub id: DbId,
    pub values: Vec<DbKeyValue>,
}
```

The `id` (i.e. `pub struct DbId(i64)`) is a numerical identifier of a database element. Positive number means the element is a `node` while negative number means the elements is an `edge`. The value `0` is a special value signifying no valid element and is used when certain queries return data not related to any particular element, e.g. aliases.

The values are `key-value` pairs (properties) associated with the given element:

```Rust
pub struct DbKeyValue {
    pub key: DbKey,
    pub value: DbValue,
}
```

The `DbKey` is an alias of `DbValue` and the value itself is an enum of valid types:

```Rust
pub enum DbValue {
    Bytes(Vec<u8>),
    Int(i64),
    Uint(u64),
    Float(DbFloat),
    String(String),
    VecInt(Vec<i64>),
    VecUint(Vec<u64>),
    VecFloat(Vec<DbFloat>),
    VecString(Vec<String>),
}
```

Note the `DbFloat` type (i.e. `pub struct DbFloat(f64)`) which is a convenient wrapper of `f64` to provide opinionated implementation of some of the operations that are not floating type friendly like comparisons. In `agdb` the float type is using [`total_cmp` standard library function](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp). Please see its documentation for important details about possible limits or issues on certain platforms.

# QueryError

Failure when running a query is reported through a single `QueryError` object which can optionally hold internal error (or chain of errors) that led to the failure. Most commonly it will represent **data error** or **logic error** in your query. Less commonly it may also report a failure to perform the requested operation due to underlying infrastructure issue (e.g. out of memory). It is up to the client code to handle the error.

# Transactions

You can run a series of queries as a transaction invoking corresponding methods on the database object:

```Rust
impl Db {
    // immutable transactions
    pub fn transaction<T, E>(&self, f: impl Fn(&Transaction) -> Result<T, E>) -> Result<T, E>

    // mutable transactions
    pub fn transaction_mut<T, E: From<QueryError>>(&mut self, f: impl Fn(&mut TransactionMut) -> Result<T, E>) -> Result<T, E>
}
```

The transaction methods take a closure that itself takes a transaction object as an argument. This is to prevent long lived transactions and force them to be as concise as possible. The transaction objects implement much the same methods as the `Db` itself (`exec` / `exec_mut`). It is not possible to nest transactions but you can run immutable queries within a mutable transaction `TransactionMut`.

Note that you cannot manually abort, rollback or commit the transaction. These are handled by the database itself based on the result of the closure. If it's `Ok` the transaction will be committed (in case `mutable` queries as there is nothing to commit for `immutable` queries). If the result is `Err` the transaction will be rolled back.

In both cases the result will be returned and the signature of the transaction methods allows for custom mapping of the default `Result<QueryResult, QueryError>` to an arbitrary `<T, E>` result-error pair.

Worth noting is that regular `exec / exec_mut` methods on the `Db` object are actually implemented as transactions.

# QueryIds & QueryId

Most queries operate over a set of database ids. The `QueryIds` type is actually an enum:

```Rust
pub enum QueryIds {
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}
```

It represents either a set of actual `ids` or a `search` query that will be executed as the larger query and its results fed as ids to the larger query. The `QueryId` is defined as another enum:

```Rust
pub enum QueryId {
    Id(DbId),
    Alias(String),
}
```

This is because you can refer to the database elements via their numerical identifier or by the `string` alias (name). The `DbId` is then just a wrapper type: `pub struct DbId(pub i64)`. Both `QueryIds` and `QueryId` can be constructed from large number of different types like raw `i64`, `&str`, `String` or vectors of those etc.

# QueryValues

The `QueryValues` is a an enum type that makes a distinction between singular and multiple values like so:

```Rust
pub enum QueryValues {
    Single(Vec<DbKeyValue>),
    Multi(Vec<Vec<DbKeyValue>>),
}
```

This is especially important because it can change the meaning of query making use of this type. For example when inserting elements into the database and supplying `QueryValues::Single` all the elements will have the copy of the single set of properties associated with them. Conversely `QueryValues::Multi` will initialize each element with a different provided set of properties bu the number of inserted elements and the number of property sets must then match (it would be a query logic error if they did not match and the query would fail with such an error).

# Mutable queries

Mutable queries are the way to modify the data in the database. Remember there can only be a mutable query running against the database at any one time preventing all other mutable or immutable queries running concurrently. There are two types of mutable queries:

- insert
- remove

The `insert` queries are used for both insert and updating data while `remove` queries are used to delete data from the database.

## Insert

There are 4 distinct insert queries:

- insert nodes
- insert edges
- insert aliases
- insert values

### Insert nodes

```Rust
pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}
```

Builder pattern:

```Rust
QueryBuilder::insert().nodes().count(2).query();
QueryBuilder::insert().nodes().count(2).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().nodes().aliases(vec!["a", "b"]).query();
QueryBuilder::insert().nodes().aliases(vec!["a", "b"]).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query();
QueryBuilder::insert().nodes().aliases(vec!["a", "b"]).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().nodes().values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query();
```

The `count` is the number of nodes to be inserted into the database. It can be omitted (left `0`) if either `values` or `aliases` (or both) are provided. If the `values` is [`QueryValues::Single`](#queryvalues) you must provide either `count` or `aliases`. It is a logic error if the count cannot be inferred and is set to `0`. If both `values` [`QueryValues::Multi`](#queryvalues) and `aliases` are provided their lengths must match, otherwise it will result in a logic error. Empty alias (`""`) are not allowed.

The result will contain:

- number of nodes inserted
- list of elements inserted with their ids (positive) but without the inserted values or aliases

### Insert edges

```Rust
pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub values: QueryValues,
    pub each: bool,
}
```

Builder pattern:

```Rust
QueryBuilder::insert().edges().from(1).to(2).query();
QueryBuilder::insert().edges().from("a").to("b").query();
QueryBuilder::insert().edges().from("a").to(vec![1, 2]).query();
QueryBuilder::insert().edges().from(vec![1, 2]).to(vec![2, 3]).query();
QueryBuilder::insert().edges().from(vec![1, 2]).to(vec![2, 3]).each().query();
QueryBuilder::insert().edges().from("a").to(vec![1, 2]).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query();
QueryBuilder::insert().edges().from("a").to(vec![1, 2]).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().edges().from_search(QueryBuilder::search().from("a").where_().node().query()).to_search(QueryBuilder::search().from("b").where_().node().query()).query();
QueryBuilder::insert().edges().from_search(QueryBuilder::search().from("a").where_().node().query()).to_search(QueryBuilder::search().from("b").where_().node().query()).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query();
QueryBuilder::insert().edges().from_search(QueryBuilder::search().from("a").where_().node().query()).to_search(QueryBuilder::search().from("b").where_().node().query()).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query();
```

The `from` and `to` represents list of origins and destinations of the edges to be inserted. As per [`QueryIds`](#queryids--queryid) it can be a list, single value, search query or even a result of another query (e.g. [insert nodes](#insert-nodes)) through the call of convenient `QueryResult::ids()` method. All ids must be `node`s and all must exist in the database otherwise data error will occur. If the `values` is [`QueryValues::Single`](#queryvalues) all edges will be associated with the copy of the same properties. If `values` is [`QueryValues::Multi`](#queryvalues) then the number of edges being inserted must match the provided values otherwise a logic error will occur. By default the `from` and `to` are expected to be of equal length specifying at each index the pair of nodes to connect with an edge. If all-to-all is desired set the `each` flag to `true`. The rule about the `values` [`QueryValues::Multi`](#queryvalues) still applies though so there must be enough values for all nodes resulting from the combination.

The result will contain:

- number of edges inserted
- list of elements inserted with their ids (negative) but without the inserted values

### Inserted aliases

```Rust
pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}
```

Builder pattern:

```Rust
QueryBuilder::insert().aliases("a").of(1).query();
QueryBuilder::insert().aliases("a").of("b").query();
QueryBuilder::insert().aliases(vec!["a", "b"]).of(vec![1, 2]).query();
```

Inserts or updates aliases of existing nodes (and only nodes, edges cannot have aliases) through this query. It takes `ids` [`QueryIds`](#queryids--queryid) and list of `aliases` as arguments. The number of aliases must match the `ids` (even if they are a search query). Empty alias (`""`) are not allowed.

Note that this query is used also for updating existing aliases. Byt inserting a different alias of an id that already has one the alias will be overwritten with the new one.

The result will contain:

- number of aliases inserted or updated
- empty list of elements

### Insert values

```Rust
pub struct InsertValuesQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}
```

Builder pattern:

```Rust
QueryBuilder::insert().values(vec![vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(vec![1, 2]).query();
QueryBuilder::insert().values(vec![vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).search(QueryBuilder::search().from("a").query()).query();
QueryBuilder::insert().values_uniform(vec![("k", "v").into(), (1, 10).into()]).ids(vec![1, 2]).query();
QueryBuilder::insert().values_uniform(vec![("k", "v").into(), (1, 10).into()]).search(QueryBuilder::search().from("a").query()).query();
```

Inserts or updates key-value pairs (properties) of existing elements. You need to specify the `ids` [`QueryIds`](#queryids--queryid) and the list of `values`. The `values` can be either [`QueryValues::Single`](#queryvalues) that will insert the single set of properties to all elements identified by `ids` or [`QueryValues::Multi`](#queryvalues) that will insert to each `id` its own set of properties but their number must match the number of `ids`.

Note that this query is used also for updating existing values. By inserting the same `key` its old value will be overwritten with the new one.

The result will contain:

- number of key-value pairs (properties) inserted
- empty list of elements

## Remove

There are 3 distinct remove queries:

- remove (elements)
- remove aliases
- remove values

### Remove elements

```Rust
pub struct RemoveQuery(pub QueryIds);
```

Builder pattern:

```Rust
QueryBuilder::remove().ids(1).query();
QueryBuilder::remove().ids("a").query();
QueryBuilder::remove().ids(vec![1, 2]).query();
QueryBuilder::remove().ids(vec!["a", "b"]).query();
QueryBuilder::remove().search(QueryBuilder::search().from("a").query()).query();
```

The elements identified by [`QueryIds`](#queryids--queryid) will be removed from the database if they exist. It is NOT an error if the elements to be removed do not exist in the database. All associated properties (key-value pairs) are also removed from all elements. Removing nodes will also remove all their edges (incoming and outgoing) and their properties.

The result will contain:

- negative number of elements removed (edges not explicitly listed or those listed but removed as part of one of their node's removal do not contribute to the result counter)
- empty list of elements

### Remove aliases

```Rust
pub struct RemoveAliasesQuery(pub Vec<String>);
```

Builder pattern:

```Rust
QueryBuilder::remove().aliases("a").query();
QueryBuilder::remove().aliases(vec!["a", "b"]).query();
```

The aliases listed will be removed from the database if they exist. It is NOT an error if the aliases do not exist in the database.

The result will contain:

- negative number of aliases removed
- empty list of elements

### Remove values

```Rust
pub struct SelectValuesQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}

pub struct RemoveValuesQuery(pub SelectValuesQuery);
```

Builder pattern:

```Rust
QueryBuilder::remove().values(vec!["k1".into(), "k2".into()]).ids(vec![1, 2]).query();
QueryBuilder::remove().values(vec!["k1".into(), "k2".into()]).search(QueryBuilder::search().from("a").query()).query();
```

The properties (key-value pairs) identified by `keys` and associated with `ids` [`QueryIds`](#queryids--queryid) will be removed from the database if they exist. It is a data error if any of the `ids` do not exist in the database but it is NOT an error if any of the keys does not exist or is not associated as property to any of the `ids`.

The result will contain:

- Number of actually removed key-value pairs
- empty list of elements

# Immutable queries

Immutable queries read the data from the database and there can be unlimited number of concurrent queries running against the database at the same time. There are two types of immutable queries:

- select
- search

The `select` queries are used to read the data from the database using known `id`s of elements. The `search` queries are used to find the `id`s and the result of search queries is thus often combined with the `select` queries.

## Select

There are 6 select queries:

- select (elements)
- select values
- select keys
- select key count
- select aliases
- select all aliases

### Select elements

```Rust
pub struct SelectQuery(pub QueryIds);
```

Builder pattern:

```Rust
QueryBuilder::select().ids("a").query();
QueryBuilder::select().ids(vec![1, 2]).query();
QueryBuilder::select().search(QueryBuilder::search().from(1).query()).query();
```

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with all their properties. If any of the ids does not exist in the database running the query will return an error. The search query is most commonly used to find, filter or otherwise limit what elements to select.

The result will contain:

- number of returned elements
- list of elements with all properties

### Select values

```Rust
pub struct SelectValuesQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}
```

Builder pattern:

```Rust
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).ids("a").query();
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).ids(vec![1, 2]).query();
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).search(QueryBuilder::search().from(1).query()).query();
```

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with only selected properties (identified by the list of keys). If any of the ids does not exist in the database or does not have all the keys associated with it then running the query will return an error. While the search query is most commonly used to find, filter or otherwise limit what elements to select, using this particular query can limit what properties will be returned.

The result will contain:

- number of returned elements
- list of elements with only selected properties

### Select keys

```Rust
pub struct SelectKeysQuery(pub QueryIds);
```

Builder pattern:

```Rust
QueryBuilder::select().keys().ids("a").query();
QueryBuilder::select().keys().ids(vec![1, 2]).query();
QueryBuilder::select().keys().search(QueryBuilder::search().from(1).query()).query();
```

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with only keys returned. If any of the ids does not exist in the database running the query will return an error. This query is most commonly used for establishing what data is available in on the graph elements (e.g. when transforming the data into a table this query could be used to populate the column names).

The result will contain:

- number of returned elements
- list of elements with only keys and default (empty `Int(0)` values)

### Select key count

```Rust
pub struct SelectKeyCountQuery(pub QueryIds);
```

Builder pattern:

```Rust
QueryBuilder::select().key_count().ids("a").query();
QueryBuilder::select().key_count().ids(vec![1, 2]).query();
QueryBuilder::select().key_count().search(QueryBuilder::search().from(1).query()).query();
```

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with only key count returned. If any of the ids does not exist in the database running the query will return an error. This query is most commonly used for establishing how many properties there are associated with the graph elements.

The result will contain:

- number of returned elements
- list of elements each with a single property (`String("key_count")`: `u64`)

### Select aliases

```Rust
pub struct SelectAliasesQuery(pub QueryIds);
```

Builder pattern:

```Rust
QueryBuilder::select().aliases().ids(vec![1, 2]).query();
QueryBuilder::select().aliases().search(QueryBuilder::search().from(1).query()).query();
```

Selects aliases of the `ids` [`QueryIds`](#queryids--queryid) or a search. If any of the ids does not have an alias running the query will return an error.

The result will contain:

- number of returned elements
- list of elements each with a single property (`String("alias")`: `String`)

### Select all aliases

```Rust
pub struct SelectAllAliases {}
```

Builder pattern:

```Rust
QueryBuilder::select().aliases().query()
```

Selects all aliases in the database.

The result will contain:

- number of elements with aliases
- list of elements with an alias each with a single property (`String("alias"): String`)

## Search

There is only a single search query.
