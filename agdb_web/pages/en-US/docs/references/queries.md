---
title: "Queries"
description: "Queries, Agnesoft Graph Database"
---

# Queries

```mermaid
flowchart LR

    QueryBuilder["<a href='https://docs.rs/agdb/latest/agdb/struct.QueryBuilder.html'>QueryBuilder</a>"] --> insert("<a href='#insert'>insert</a>")
    QueryBuilder --> remove("<a href='#remove'>remove</a>")
    QueryBuilder --> select("<a href='#select'>select</a>")
    QueryBuilder --> search("<a href='#search'>search</a>")

    insert --> i_aliases("<a href='#insert-aliases'>aliases</a>") --> i_a_ids("ids") --> InsertAliasesQuery["<a href='#insert-aliases'>InsertAliasesQuery</a>"]
    insert --> i_edges("<a href='#insert-edges'>edges</a>") --> i_e_from("from") --> i_e_to("to") --> InsertEdgesQuery["<a href='#insert-edges'>InsertEdgesQuery</a>"]
    i_edges --> i_e_ids("ids")
    i_e_ids --> i_e_from
    i_e_to --> each("each") --> InsertEdgesQuery
    i_e_to --> i_e_values("values")
    each --> i_e_values_uniform("values_uniform") --> InsertEdgesQuery
    each --> i_e_values("values") --> InsertEdgesQuery
    insert --> i_index("<a href='#insert-index'>index</a>") --> InsertIndexQuery["<a href='#insert-index'>InsertIndexQuery</a>"]
    insert --> i_nodes("<a href='#insert-nodes'>nodes</a>")
    i_nodes --> i_n_values("values") --> InsertNodesQuery["<a href='#insert-nodes'>InsertNodesQuery</a>"]
    i_nodes --> i_n_aliases("aliases")
    i_nodes --> i_n_ids("ids")
    i_n_count --> i_n_values_uniform("values_uniform")
    i_n_aliases --> i_n_values
    i_n_aliases --> InsertNodesQuery
    i_n_aliases --> i_n_values_uniform
    i_n_ids --> i_n_values
    i_n_ids --> i_n_aliases
    i_n_ids --> i_n_count
    i_nodes --> i_n_count("count") --> InsertNodesQuery
    insert --> i_element("<a href='#insert-values'>element</a>") --> InsertValuesQuery["<a href='#insert-values'>InsertValuesQuery</a>"]
    insert --> i_elements("<a href='#insert-values'>elements</a>") --> InsertValuesQuery
    insert --> i_values("<a href='#insert-values'>values</a>")
    i_values --> i_v_ids("ids") --> InsertValuesQuery
    insert --> i_values_uniform("<a href='#insert-values'>values_uniform</a>") --> InsertValuesQuery

    remove --> r_aliases("<a href='#remove-aliases'>aliases</a>") --> RemoveAliasesQuery["<a href='#remove-aliases'>RemoveAliasesQuery</a>"]
    remove --> r_ids("<a href='#remove-elements'>ids</a>") --> RemoveQuery["<a href='#remove-elements'>RemoveQuery</a>"]
    remove --> r_index("<a href='#remove-index'>index</a>") --> RemoveIndexQuery["<a href='#remove-index'>RemoveIndexQuery</a>"]
    remove --> r_values("<a href='#remove-values'>values</a>") --> r_v_ids("ids") --> RemoveValuesQuery["<a href='#remove-values'>RemoveValuesQuery</a>"]

    select --> s_aliases("<a href='#select-aliases'>aliases</a>") --> SelectAllAliasesQuery["<a href='#select-all-aliases'>SelectAllAliasesQuery</a>"]
    s_aliases --> s_a_ids("ids") --> SelectAliasesQuery["<a href='#select-aliases'>SelectAliasesQuery</a>"]
    select --> s_ids("<a href='#select-values'>ids</a>") --> SelectValuesQuery["<a href='#select-values'>SelectValuesQuery</a>"]
    select --> s_indexes("<a href='#select-indexes'>indexes</a>") --> SelectIndexesQuery["<a href='#select-indexes'>SelectIndexesQuery</a>"]
    select --> s_keys("<a href='#select-keys'>keys</a>") --> s_k_ids("ids") --> SelectKeysQuery["<a href='#select-keys'>SelectKeysQuery</a>"]
    select --> key_count("<a href='#select-key-count'>key_count</a>") --> s_k_c_ids("ids") --> SelectKeyCountQuery["<a href='#select-key-count'>SelectKeyCountQuery</a>"]
    select --> edge_count("<a href='#select-edge-count'>edge_count</a>") ---> s_e_c_ids("ids") ---> SelectEdgeCountQuery["<a href='#select-edge-count'>SelectEdgeCountQuery</a>"]
    select --> edge_count_from("<a href='#select-edge-count'>edge_count</a>") ---> s_e_c_ids("ids")
    select --> edge_count_to("<a href='#select-edge-count'>edge_count</a>") ---> s_e_c_ids("ids")
    select --> select_node_count("<a href='#select-node-count'>node_count</a>")
    select --> values("<a href='#select-values'>values</a>") --> s_v_ids("ids") --> SelectValuesQuery["<a href='#select-values'>SelectValuesQuery</a>"]
    select --> elements("<a href='#select-values'>values</a>") --> s_v_ids("ids")

    search --> index("index") --> s_i_value("value") --> SearchQuery["<a href='#search'>SearchQuery</a>"]
    search --> from("from") --> SearchQuery
    search --> elements("elements") --> to
    from --> limit("limit") --> SearchQuery
    from --> offset("offset")
    offset --> limit
    from --> order_by("order_by")
    order_by --> offset
    order_by --> limit
    order_by --> SearchQuery
    from --> where --> SearchQuery
    from --> to("to")
    to --> order_by
    to --> offset
    to --> limit
    search --> breadth_first("breadth_first") --> from
    search --> depth_first("depth_first") --> from
    search --> to
    depth_first --> to
    breadth_first --> to
    to --> where(("<a href='#conditions'>where</a>"))
    order_by --> where
    offset --> where
    limit --> where

    condition --> SearchQuery
    end_where("end_where") --> SearchQuery
    where --> condition
    modifier --> where
    condition --> end_where
    end_where --> logic
    where --> modifier("not/beyond")
    modifier --> condition[["<a href='#conditions'>distance<br/>edge<br/>edge_count<br/>edge_count_from<br/>edge_count_to<br/>ids<br/>key.value<br/>keys<br/>node</a>"]]
    condition --> logic("and/or")
    logic --> where
```

All interactions with the `agdb` are realized through queries. There are two kinds of queries:

- Immutable queries
- Mutable queries

Immutable queries read the data from the database through `select` and `search` queries. Mutable queries write to or delete from the database through `insert` and `remove` queries. All queries follow the Rust rules about borrowing:

```
There can be unlimited number of immutable concurrent queries or exactly one mutable query running against the database.
```

The queries are executed against the database by calling the corresponding method on the database object:

```rs
impl Db {
    // immutable queries only
    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError>

    // mutable queries only
    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError>
}
```

Alternatively you can run a series of queries as a [transaction](#transactions).

All queries return `Result<QueryResult, QueryError>`. The [`QueryResult`](#queryresult) is the universal data structure holding results of all queries in a uniform structure. The [`QueryError`](#queryerror) is the singular error type holding information of any failure or problem encountered when running the query.

## Types

### DbUserValue

The `DbUserValue` trait is an interface that can be implemented for user defined types so that they can be seamlessly used with the database:

```rs
pub trait DbUserValue: Sized {
    fn db_id(&self) -> Option<DbId>;
    fn db_keys() -> Vec<DbValue>;
    fn from_db_element(element: &DbElement) -> Result<Self, DbError>;
    fn to_db_values(&self) -> Vec<DbKeyValue>;
}
```

Typically, you would derive this trait with `agdb::UserValue` procedural macro that uses the field names as keys (of type `String`) and losslessly converts the values when reading/writing from/to the database from supported types (e.g. field type `i32` will become `i64` in the database).

It is recommended but optional to have `db_id` field of type `Option<T: Into<DbId>>` (e.g. `Option<QueryId>` or `Option<DbId>`) in your user defined types which will further allow you to directly update your values with query shorthands. However, it is optional, and all other features will still work including conversion from `QueryResult` or passing your types to `values()` in the builders.

The `agdb::UserValue` macro also supports `Option`al types. When a value is `None` it will be omitted when saving the object to the database.

Example:

```rs
#[derive(UserValue)]
struct User { db_id: Option<DbId>, name: String, }
let user = User { db_id: None, name: "Bob".to_string() };
db.exec_mut(QueryBuilder::insert().nodes().values(user).query())?;
let mut user: User = db.exec(QueryBuilder::select().values(User::db_keys()).ids(1).query())?.try_into()?; // User { db_id: Some(DbId(1)), name: "Bob" }
user.name = "Alice".to_string();
db.exec_mut(QueryBuilder::insert().element(&user).query())?; //updates the user element with new name
```

In some cases you may want to implement the `DbUserValue` trait yourself. For example when you want to omit a field entirely or construct it based on other values.

Types not directly used in the database but for which the conversions are supported:

- u32 <=> u64
- i32 <=> i64
- f32 <=> f64
- Vec<i32> <=> Vec<i64>
- Vec<u32> <=> Vec<u64>
- Vec<f32> <=> Vec<f64>
- &str => String (only one way conversion to `String`)
- Vec<&str> => Vec<String> (only one way conversion to `Vec<String>`)
- bool (\*)

\* The boolean type is not a native type in the `agdb`, but you can still use it in your types in any language. The `bool` type will be converted to `u64` (0 == false, 1 == true). The `Vec<bool>` type will be converted to `Vec<u8>` (bytes, 0 == false, 1 == true). The conversion back to `bool` is possible from wider range of values - the same rules apply for vectorized version which however cannot be converted to from single values:

- u64 / i64: any non-zero value will be `true`
- f64: any value except `0.0` will be `true`
- string: only `"true"` or `"1"` will be `true`

### QueryResult

The `QueryResult` is the universal result type for all successful queries. It can be converted to user defined types that implement [`DbUserValue`](#dbuservalue) with `try_into()`. It looks like this:

```rs
pub struct QueryResult {
    pub result: i64,
    pub elements: Vec<DbElement>,
}
```

The `result` field holds numerical result of the query. It typically returns the number of database items affected. For example when selecting from the database it will hold a positive number of elements returned. When removing from the database it will hold a negative number of elements deleted from the database. The optional `from` and `to` fields will hold origin/destination `id` of an edge and will be `None` for nodes.

The `elements` field hold the [database elements](/docs/guides/concepts#graph) returned. Each element looks like:

```rs
pub struct DbElement {
    pub id: DbId,
    pub from: Option<DbId>,
    pub to: Option<DbId>,
    pub values: Vec<DbKeyValue>,
}
```

The `id` (i.e. `pub struct DbId(i64)`) is a numerical identifier of a database element. Positive number means the element is a `node` while negative number means the elements is an `edge`. The value `0` is a special value signifying no valid element and is used when certain queries return data not related to any particular element, e.g. aliases.

The values are `key-value` pairs (properties) associated with the given element:

```rs
pub struct DbKeyValue {
    pub key: DbValue,
    pub value: DbValue,
}
```

Where `DbValue` is:

```rs
pub enum DbValue {
    Bytes(Vec<u8>),
    I64(i64),
    U64(u64),
    F64(DbF64),
    String(String),
    VecI64(Vec<i64>),
    VecU64(Vec<u64>),
    VecF64(Vec<DbF64>),
    VecString(Vec<String>),
}
```

Note the `DbF64` type (i.e. `pub struct DbF64(f64)`) which is a convenient wrapper of `f64` to provide opinionated implementation of some of the operations that are not floating type friendly like comparisons. In `agdb` the float type is using [`total_cmp` standard library function](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp). Please see its documentation for important details about possible limits or issues on certain platforms.

The enum variants can be conveniently accessed through methods named after each variant:

```rs
fn bytes(&self) -> Result<&Vec<u8>, DbError>;
fn to_bool(&self) -> Result<bool, DbError>;
fn to_f64(&self) -> Result<DbF64, DbError>;
fn to_i64(&self) -> Result<i64, DbError>;
fn to_u64(&self) -> Result<u64, DbError>;
fn to_string(&self) -> String;
fn string(&self) -> Result<&String, DbError>;
fn vec_f64(&self) -> Result<&Vec<DbF64>, DbError>;
fn vec_i64(&self) -> Result<&Vec<i64>, DbError>;
fn vec_u64(&self) -> Result<&Vec<u64>, DbError>;
fn vec_string(&self) -> Result<&Vec<String>, DbError>;
fn vec_bool(&self) -> Result<Vec<bool>, DbError>;
```

The numerical variants (`I64`, `U64`, `DbF64`) will attempt loss-less conversions where possible. To avoid copies all other variants return `&` where conversions are not possible even if they could be done in theory. The special case is `to_string()` provided by the `Display` trait. It converts any values into string (it also copies the `String` variant) and performs possibly lossy conversion from `Bytes` to UTF-8 string. For `bool` conversion details refer to [DbUserValue](#dbuservalue) section.

### QueryError, DbError

Failure when running a query is reported through a single `QueryError` object which can optionally hold internal error (or chain of errors) that led to the failure. Most commonly it will represent **data error** or **logic error** in your query. Less commonly it may also report a failure to perform the requested operation due to underlying infrastructure issue (e.g. out of memory) in which case the nested error would be of type `DbError`. It is up to the client code to handle the errors.

### QueryId, QueryIds

Most queries operate over a set of database `ids`. The `QueryIds` type is actually an enum:

```rs
pub enum QueryIds {
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}
```

It represents either a set of actual `ids` or a `search` query that will be executed as the larger query and its results fed as `ids` to the larger query. The `QueryId` is defined as another enum:

```rs
pub enum QueryId {
    Id(DbId),
    Alias(String),
}
```

This is because you can refer to the database elements via their numerical identifier or by the `string` alias (name). The `DbId` is then just a wrapper type: `pub struct DbId(pub i64)`. Both `QueryIds` and `QueryId` can be constructed from large number of different types like raw `i64`, `&str`, `String` or vectors of those etc.

### QueryValues

The `QueryValues` is an enum type that makes a distinction between singular and multiple values like so:

```rs
pub enum QueryValues {
    Single(Vec<DbKeyValue>),
    Multi(Vec<Vec<DbKeyValue>>),
}
```

This is especially important because it can change the meaning of a query making use of this type. For example when inserting elements into the database and supplying `QueryValues::Single` all the elements will have the copy of the single set of properties associated with them. Conversely, `QueryValues::Multi` will initialize each element with a different provided set of properties but the number of inserted elements and the number of property sets must then match (it would be a query logic error if they did not match and the query would fail with such an error).

## Mutable queries

Mutable queries are the way to modify the data in the database. Remember there can only be a mutable query running against the database at any one time preventing all other mutable or immutable queries running concurrently. There are two types of mutable queries:

- insert
- remove

The `insert` queries are used for both insert and updating data while `remove` queries are used to delete data from the database.

## Immutable queries

Immutable queries read the data from the database and there can be an unlimited number of concurrent queries running against the database at the same time. There are two types of immutable queries:

- select
- search

The `select` queries are used to read the data from the database using known `id`s of elements. The `search` queries are used to find the `id`s and the result of search queries is thus often combined with the `select` queries.

## Transactions

You can run a series of queries as a transaction invoking corresponding methods on the database object:

```rs
impl Db {
    // immutable transaction
    pub fn transaction<T, E>(&self, mut f: impl FnMut(&Transaction) -> Result<T, E>) -> Result<T, E>

    // mutable transaction
    pub fn transaction_mut<T, E: From<QueryError>>(&mut self, mut f: impl FnMut(&mut TransactionMut) -> Result<T, E>) -> Result<T, E>
}
```

The transaction methods take a closure that itself takes a transaction object as an argument. This is to prevent long-lived transactions and force them to be as concise as possible. The transaction objects implement the same execution methods as the `Db` itself (`exec` / `exec_mut`). It is not possible to nest transactions, but you can run immutable queries within a mutable transaction `TransactionMut`.

Note that you cannot manually abort, rollback or commit the transaction. These are handled by the database itself based on the result of the closure. If it's `Ok` the transaction will be committed (in case of the `mutable` queries as there is nothing to commit for `immutable` queries). If the result is `Err` the transaction will be rolled back.

In both cases the result will be returned and the signature of the transaction methods allows for custom mapping of the default `Result<QueryResult, QueryError>` to an arbitrary `<T, E>` result-error pair.

Worth noting is that regular `exec / exec_mut` methods on the `Db` object are actually implemented as transactions.

## Insert

There are 5 distinct insert queries:

- insert aliases
- insert edges
- insert nodes
- insert index
- insert values

### Insert aliases

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of inserted/updated aliases
    pub elements: Vec<DbElement>, // empty
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::insert().aliases("a").ids(1).query();
QueryBuilder::insert().aliases("a").ids("b").query(); // alias "b" is replaced  with "a"
QueryBuilder::insert().aliases(["a", "b"]).ids([1, 2]).query();
```

</td></tr></table>

Inserts or updates aliases of existing nodes (and only nodes, edges cannot have aliases) through this query. It takes `ids` [`QueryIds`](#queryids--queryid) and list of `aliases` as arguments. The number of aliases must match the `ids` (even if they are a search query). Empty alias (`""`) are not allowed.

Note that this query is also used for updating existing aliases. By inserting a different alias of an `id` that already has one that alias will be overwritten with the new one.

### Insert edges

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub ids: QueryIds,
    pub values: QueryValues,
    pub each: bool,
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of inserted edges
    pub elements: Vec<DbElement>, // list of inserted edges (only ids)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::insert().edges().from(1).to(2).query();
QueryBuilder::insert().edges().from("a").to("b").query();
QueryBuilder::insert().edges().from("a").to([1, 2]).query();
QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).query();
QueryBuilder::insert().edges().from([1, 2]).to([2, 3]).each().query();
QueryBuilder::insert().edges().from("a").to([1, 2]).values([[("k", 1).into()], [("k", 2).into()]]).query();
QueryBuilder::insert().edges().from("a").to([1, 2]).values_uniform([("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query();
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values([[("k", 1).into()], [("k", 2).into()]]).query();
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform([("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().edges().ids(-3).from(1).to(2).query();
QueryBuilder::insert().edges().ids([-3, -4]).from(1).to(2).query();
QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query()).from(1).to(2).query();
```

</td></tr></table>

The `from` and `to` represents list of origins and destinations of the edges to be inserted. As per [`QueryIds`](#queryids--queryid) it can be a list, single value, search query or even a result of another query (e.g. [insert nodes](#insert-nodes)) through the call of convenient `QueryResult::ids()` method. All `ids` must be `node`s and all must exist in the database otherwise data error will occur. If the `values` is [`QueryValues::Single`](#queryvalues) all edges will be associated with the copy of the same properties. If `values` is [`QueryValues::Multi`](#queryvalues) then the number of edges being inserted must match the provided values otherwise a logic error will occur. By default, the `from` and `to` are expected to be of equal length specifying at each index the pair of nodes to connect with an edge. If all-to-all is desired set the `each` flag to `true`. The rule about the `values` [`QueryValues::Multi`](#queryvalues) still applies though so there must be enough values for all nodes resulting from the combination. The values can be inferred from user defined types if they implement `DbUserValue` trait (`#derive(agdb::UserValue)`). Both singular and vectorized versions are supported. Optionally one can specify `ids` that facilitates insert-or-update semantics. The field can be a search sub-query. If the resulting list in `ids` is empty the query will insert edges as normal. If the list is not empty all `ids` must exist and refer to existing edges and the query will perform update of values instead. Note: the specified from/to (origin/destination) for the updated edges is not checked against those supplied via `ids`.

### Insert index

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct InsertIndexQuery(pub DbValue);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of indexed values
    pub elements: Vec<DbElement>, // empty
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::insert().index("key").query();
```

</td></tr></table>

Creates an index for a key. The index is valid for the entire database including any and all existing values in the database. The purpose of the index is to provide faster lookup for data that is not modelled on the graph itself. Example can be looking up users by their username or token.

### Insert nodes

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
    pub ids: QueryIds,
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of inserted nodes
    pub elements: Vec<DbElement>, // list of inserted nodes (only ids)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::insert().nodes().count(2).query();
QueryBuilder::insert().nodes().count(2).values_uniform([("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().nodes().aliases(["a", "b"]).query();
QueryBuilder::insert().nodes().aliases(["a", "b"]).values([[("k", 1).into()], [("k", 2).into()]]).query();
QueryBuilder::insert().nodes().aliases(["a", "b"]).values_uniform([("k", "v").into(), (1, 10).into()]).query();
QueryBuilder::insert().nodes().values([[("k", 1).into()], [("k", 2).into()]]).query();
QueryBuilder::insert().nodes().ids(1).count(1).query();
QueryBuilder::insert().nodes().ids([1, 2]).count(1).query();
QueryBuilder::insert().nodes().ids("a").count(1).query();
QueryBuilder::insert().nodes().ids("a").aliases("a").query(),
QueryBuilder::insert().nodes().ids(["a", "b"]).count(1).query();
QueryBuilder::insert().nodes().ids([1, 2]).values([[("k", "v").into()], [(1, 10).into()]]).query(),
QueryBuilder::insert().nodes().ids([1, 2]).values_uniform([("k", "v").into(), (1, 10).into()]).query(),
QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query()).count(1).query();
```

</td></tr></table>

The `count` is the number of nodes to be inserted into the database. It can be omitted (left `0`) if either `values` or `aliases` (or both) are provided. If the `values` is [`QueryValues::Single`](#queryvalues) you must provide either `count` or `aliases`. It is not an error if the count is set to `0`, but the query will be a no-op and return empty result.

If both `values` [`QueryValues::Multi`](#queryvalues) and `aliases` are provided their lengths must be compatible (aliases <= values), otherwise it will result in a logic error. Empty aliases (`""`) are not allowed. The values can be inferred from user defined types if they implement `DbUserValue` trait (`#derive(agdb::UserValue)`). Both singular and vectorized versions are supported. Optionally one can specify `ids` that facilitates insert-or-update semantics. The field can be a search sub-query. If the resulting list in `ids` is empty the query will insert nodes as normal.

If the list is not empty all `ids` must exist and must refer to nodes and the query will perform update instead - both aliases (replacing existing ones if applicable) and values.

If an alias already exists in the database its values will be amended (inserted or replaced) with the provided values.

### Insert values

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct InsertValuesQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of inserted key-value pairs
    pub elements: Vec<DbElement>, // list of new elements
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::insert().element(&T { ... }).query(); //Where T: DbUserValue (i.e. #derive(UserValue))
QueryBuilder::insert().elements(&vec![T {...}, T {...}]).query(); //Where T: DbUserValue (i.e. #derive(UserValue))
QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids([1, 2]).query();
QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::insert().values([vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).search().from("a").query(); //Equivalent to the previous query
QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).ids([1, 2]).query();
QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::insert().values_uniform([("k", "v").into(), (1, 10).into()]).search().from("a").query(); //Equivalent to the previous query
```

</td></tr></table>

Inserts or updates key-value pairs (properties) of existing elements or insert new elements (nodes). You need to specify the `ids` [`QueryIds`](#queryids--queryid) and the list of `values`. The `values` can be either [`QueryValues::Single`](#queryvalues) that will insert the single set of properties to all elements identified by `ids` or [`QueryValues::Multi`](#queryvalues) that will insert to each `id` its own set of properties, but their number must match the number of `ids`. If the user defined type contains `db_id` field of type `Option<T: Into<QueryId>>` you can use the shorthand `insert().element() / .insert().elements()` that will infer the values and `ids` from your types. The `values()` will be inferred from user defined types if they implement `DbUserValue` trait (`#derive(agdb::UserValue)`). Both singular and vectorized versions are supported.

- If an `id` is non-0 or an existing alias that element will be updated in the database with provided values.
- If an `id` is `0` or a non-existent alias new element (node) will be inserted into the database with that alias.

Note: that this query is insert-or-update for both nodes and existing values. By inserting the same `key` its old value will be overwritten with the new one.

## Remove

There are 4 distinct remove queries:

- remove aliases
- remove (elements)
- remove index
- remove values

### Remove aliases

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct RemoveAliasesQuery(pub Vec<String>);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // negative number of removed aliases
    pub elements: Vec<DbElement>, // empty
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::remove().aliases("a").query();
QueryBuilder::remove().aliases(["a", "b"]).query();
```

</td></tr></table>

The aliases listed will be removed from the database if they exist. It is NOT an error if the aliases do not exist in the database.

### Remove elements

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct RemoveQuery(pub QueryIds);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // negative number of removed ids
                     // (does not include removed edges
                     // unless listed in query ids)
    pub elements: Vec<DbElement>, // empty
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::remove().ids(1).query();
QueryBuilder::remove().ids("a").query();
QueryBuilder::remove().ids([1, 2]).query();
QueryBuilder::remove().ids(["a", "b"]).query();
QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::remove().search().from("a").query(); //Equivalent to the previous query
```

</td></tr></table>

The elements identified by [`QueryIds`](#queryids--queryid) will be removed from the database if they exist. It is NOT an error if the elements to be removed do not exist in the database. All associated properties (key-value pairs) are also removed from all elements. Removing nodes will also remove all their edges (incoming and outgoing) and their properties.

### Remove index

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct RemoveIndexQuery(pub DbValue);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // negative number of values removed
                     // from the index
    pub elements: Vec<DbElement>, // empty
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::remove().index("key").query();
```

</td></tr></table>

Removes an index from the database. It is NOT an error if the index does not exist in the database.

### Remove values

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct RemoveValuesQuery(pub SelectValuesQuery);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // negative number of actually removed
                     // key-value pairs
    pub elements: Vec<DbElement>, // empty
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::remove().values(["k1".into(), "k2".into()]).ids([1, 2]).query();
QueryBuilder::remove().values(["k1".into(), "k2".into()]).ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::remove().values(["k1".into(), "k2".into()]).search().from("a").query(); //Equivalent to the previous query
```

</td></tr></table>

NOTE: See [`SelectValuesQuery`](#select-values) for more details.

The properties (key-value pairs) identified by `keys` and associated with `ids` [`QueryIds`](#queryids--queryid) will be removed from the database if they exist. It is an error if any of the `ids` do not exist in the database, but it is NOT an error if any of the keys does not exist or is not associated as property to any of the `ids`.

## Select

There are following select queries:

- select aliases
- select all aliases
- select edge count
- select (elements)
- select indexes
- select keys
- select key count
- select values

### Select aliases

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectAliasesQuery(pub QueryIds);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of returned elements
    pub elements: Vec<DbElement>, // list of elements each with
                                  // a single property
                                  // (`String("alias")`: `String`)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().aliases().ids([1, 2]).query();
QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query();
QueryBuilder::select().aliases().search().from(1).query(); //Equivalent to the previous query
```

</td></tr></table>

Selects aliases of the `ids` [`QueryIds`](#queryids--queryid) or a search. If any of the `ids` does not have an alias running the query will return an error.

### Select all aliases

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectAllAliasesQuery {}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of elements with aliases
    pub elements: Vec<DbElement>, // list of elements with an
                                  // alias each with a single
                                  // property (`String("alias"): String`)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().aliases().query();
```

</td></tr></table>

Selects all aliases in the database.

### Select edge count

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectEdgeCountQuery {
    pub ids: Ids,
    pub from: bool,
    pub to: bool
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of elements with aliases
    pub elements: Vec<DbElement>, // list of elements with an
                                  // alias each with a single
                                  // property (`String("edge_count"): String`)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().edge_count().ids([1, 2]).query();
QueryBuilder::select().edge_count_from().ids([1, 2]).query();
QueryBuilder::select().edge_count_to().ids([1, 2]).query();
QueryBuilder::select().edge_count().ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::select().edge_count().search().from("a").query(); // Equivalent to the previous query
```

</td></tr></table>

Selects count of edges of nodes (`ids`). The `edge_count` variant counts all edges (outgoing & incoming). The `edge_count_from` counts only outgoing edges. The `edge_count_to` counts only incoming edges.

NOTE: Self-referential edges (going from the same node to the same node) will be counted twice in the first variant (`edge_count`) as the query counts outgoing/incoming edges rather than unique database elements. As a result the `edge_count` result may be higher than the actual number of physical edges in such a case.

### Select indexes

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectIndexesQuery {};
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of indexes in the database
    pub elements: Vec<DbElement>, // single element with id 0 and list of
                                  // properties representing each index
                                  // (`DbValue`: `u64`) where the key is
                                  // the indexed key and the value is number
                                  // of indexed values in the index.
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().indexes().query();
```

</td></tr></table>

Selects all indexes in the database.

### Select keys

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectKeysQuery(pub QueryIds);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of returned elements
    pub elements: Vec<DbElement>, // list of elements with only keys
                                  // defaulted values will be `I64(0)`
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().keys().ids("a").query();
QueryBuilder::select().keys().ids([1, 2]).query();
QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query();
QueryBuilder::select().keys().search().from(1).query(); // Equivalent to the previous query
```

</td></tr></table>

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with only keys returned. If any of the `ids` does not exist in the database running the query will return an error. This query is most commonly used for establishing what data is available in on the graph elements (e.g. when transforming the data into a table this query could be used to populate the column names).

### Select key count

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectKeyCountQuery(pub QueryIds);
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of returned elements
    pub elements: Vec<DbElement>, // list of elements each with a
                                  // single property
                                  // (`String("key_count")`: `u64`)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().key_count().ids("a").query();
QueryBuilder::select().key_count().ids([1, 2]).query();
QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query();
QueryBuilder::select().key_count().search().from(1).query(); // Equivalent to the previous query
```

</td></tr></table>

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with only key count returned. If any of the `ids` does not exist in the database running the query will return an error. This query is most commonly used for establishing how many properties there are associated with the graph elements.

### Select node count

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectNodeCountQuery {}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // Always  1
    pub elements: Vec<DbElement>, // single element with single property (`String("node_count"): String`)
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::select().node_count().query();
```

</td></tr></table>

Selects number (count) of nodes in the database.

### Select values

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SelectValuesQuery {
    pub keys: Vec<DbValue>,
    pub ids: QueryIds,
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of returned elements
    pub elements: Vec<DbElement>, // list of elements with only
                                  // selected properties
}
```

</td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs

QueryBuilder::select().ids("a").query();
QueryBuilder::select().ids([1, 2]).query();
QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query();
QueryBuilder::select().search().from(1).query(); // Equivalent to the previous query
QueryBuilder::select().values(["k".into(), "k2".into()]).ids("a").query();
QueryBuilder::select().values(["k".into(), "k2".into()]).ids([1, 2]).query();
QueryBuilder::select().values(["k".into(), "k2".into()]).ids(QueryBuilder::search().from(1).query()).query();
QueryBuilder::select().values(["k".into(), "k2".into()]).search().from(1).query(); // Equivalent to the previous query
QueryBuilder::select().elements::<T>().ids(1).query();
QueryBuilder::select().elements::<T>().ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::select().elements::<T>().search().from("a").query(); // Equivalent to the previous query
```

</td></tr></table>

Selects elements identified by `ids` [`QueryIds`](#queryids--queryid) or search query with only selected properties (identified by the list of keys). If any of the `ids` does not exist in the database or does not have all the keys associated with it then running the query will return an error. The search query is most commonly used to find, filter or otherwise limit what elements to select. You can limit what properties will be returned. If the list of properties to select is empty all properties will be returned. If you plan to convert the result into your user defined type(s) you should use either `elements::<T>()` variant or supply the list of keys to `values()` with `T::db_keys()` provided through the `DbUserValue` trait (`#derive(UserValue)`) as argument to `values()` otherwise the keys may not be in an expected order even if they are otherwise present.

## Search

<table><tr><td><b>Struct</b></td><td><b>Result</b></td></tr>
<tr><td>

```rs
pub struct SearchQuery {
    pub algorithm: SearchQueryAlgorithm,
    pub origin: QueryId,
    pub destination: QueryId,
    pub limit: u64,
    pub offset: u64,
    pub order_by: Vec<DbKeyOrder>,
    pub conditions: Vec<QueryCondition>,
}
```

</td><td>

```rs
pub struct QueryResult {
    pub result: i64, // number of elements found
    pub elements: Vec<DbElement>, // list of elements found (only ids)
}
```

</td></tr><tr><td>

```rs
pub enum SearchQueryAlgorithm {
    BreadthFirst,
    DepthFirst,
    Index,
    Elements
}

pub enum DbKeyOrder {
    Asc(DbValue),
    Desc(DbValue),
}
```

</td><td></td></tr><tr><td colspan=2><b>Builder</b></td></tr><tr><td colspan=2>

```rs
QueryBuilder::search().from("a").query();
QueryBuilder::search().to(1).query(); //reverse search
QueryBuilder::search().from("a").to("b").query(); //path search using A* algorithm
QueryBuilder::search().breadth_first().from("a").query(); //breadth first is the default and can be omitted
QueryBuilder::search().depth_first().from("a").query();
QueryBuilder::search().elements().query();
QueryBuilder::search().index("age").value(20).query(); //index search
//limit, offset and order_by can be applied similarly to all the search variants except search index
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("age".into()), DbKeyOrder::Asc("name".into())]).query()
QueryBuilder::search().from(1).offset(10).query();
QueryBuilder::search().from(1).limit(5).query();
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).query();
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).limit(5).query();
QueryBuilder::search().from(1).order_by([DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query();
QueryBuilder::search().from(1).offset(10).limit(5).query();
```

</td></tr></table>

There is only a single search query that provides the ability to search the graph or indexes. When searching the graph it examines connected elements and their properties. While it is possible to construct the search queries manually, specifying them that way can be excessively difficult and therefore **using the builder pattern is recommended**. The default search algorithm is `breadth first` however you can choose to use `depth first`. For path search the `A*` algorithm is used. For searching an index the algorithm is `index`. For searching disregarding the graph structure and indexes (full search) the algorithm is `elements`. Elements will never be examined twice during any search regardless of any cycles in the graph.

Very often you would want the values / elements to be returned from the search query. To accomplish it you need to nest the search query in the select query with either `.search()` builder element or `ids()` step that takes a `SearchQuery` as argument. That fetches the data as the search query only traverses the graph. E.g. `QueryBuilder::select().search().from("alias").query()`. Refer to the [Select Values](#select-values) query for details.

If the index search is done the graph traversal is skipped entirely as are most of the parameters including like limit, offset, ordering and conditions.

The graph search query is made up of the `origin` and `destination` of the search and the algorithm. Specifying only `origin` (from) will result in a search along `from->to` edges. Specifying only `destination` (to) will result in the reverse search along the `to<-from` edges. When both `origin` and `destination` are specified the search algorithm becomes a path search and the algorithm used will be `A*`. Optionally you can specify a `limit` (0 = unlimited) and `offset` (0 = no offset) to the returned list of graph element `ids`. If specified (!= 0) the `origin` and the `destination` must exist in the database, otherwise an error will be returned. The elements can be optionally ordered with `order_by` list of keys allowing ascending/descending ordering based on multiple properties.

When searching `elements` the database is being scanned in linearly one element (node & edge) at a time which can be very slow. Consider using `limit` in this case. However, this search can be useful in exploration, when the database structure is not known, when searching for abandoned/lost elements and other edge cases not covered by regular search algorithms. The default order of returned elements is from the lowest internal db `id` to the highest which does not necessarily indicate age of the elements as the `ids` can be reused when elements are deleted.

Finally, the list of `conditions` that each examined graph element must satisfy to be included in the result (and subjected to the `limit` and `offset`).

**NOTE:** When both `origin` and `destination` are specified, and the algorithm is switched to the `A*` the `limit` and `offset` are applied differently. In regular (open-ended) search the search will end when the `limit` is reached but with the path search (A\*) the `destination` must be reached first before they are applied.

### Breadth First

The `breadt first` algorithm (the default one) examines every element on each level before moving to the next level. For instance starting at a node this algorithm will first examine all the edges in the selected direction (from/to) before examining the adjacent nodes reachable through those edges. The order of the elements is **from newest to oldest** where newest means most recently connected. Similarly, the next level is also examined in the same order. Example:

Given a graph of 6 nodes connected together with 4 edges like so (NOTE: `ids` are for illustration only and does NOT indicate newer/older element):

| Level 0  | Level 1  | Level 2  | Level 3  | Level 4  |
| -------- | -------- | -------- | -------- | -------- |
| Node (a) | Edge (b) | Node (d) | Edge (f) | Node (h) |
|          | Edge (c) | Node (e) | Edge (g) | Node (j) |

The `breadth first` algorithm will first visit node (a) at level 0. Then it will visit all edges at level 1 starting with the newest edge (c) followed by the older edge (b). Then it will move on to the level 2 once more examining the newest node first (g) followed by (f). Lastly it will move on to level 4 examining nodes (j) and (h). The "newest" means most recently connected and does not necessarily mean it will have higher `id` because `ids` can be reused from deleted elements.

### Depth First

The `depth first` algorithm follows every element to the next level first. When it cannot continue on to a next level it will step back to the previous level trying another direction if possible. When exhausted or not available it will backtrack again to the previous level and continue from there. The order of the elements is **from newest to oldest** where newest means most recently connected. Example:

Given a graph of 6 nodes connected together with 4 edges like so (NOTE: `ids` are for illustration only and does NOT indicate newer/older element):

| Level 0  | Level 1  | Level 2  | Level 3  | Level 4  |
| -------- | -------- | -------- | -------- | -------- |
| Node (a) | Edge (b) | Node (d) |          |          |
|          | Edge (c) | Node (e) | Edge (f) | Node (h) |
|          |          |          | Edge (g) | Node (j) |

The `depth first` algorithm will first visit node (a) at level 0. Then it will visit the most recent edge (c) on level 1. Then it will follow it to its connected node (e) at level 2, then edge (g) at level 3 and finally node (j) at level 4. Since it cannot continue it will step back to level 3 and examine the edge (f) and follow it to node (h) at level 4. After that it will step back to level 3 and see nothing available so it will backtrack further to level 1 to examine edge (b) and follow it to node (d). That will conclude the search.

NOTE: when a graph contains multiple edges leading to the same elements the extra edges will appear seemingly "out of order" in the search result (i.e. at the end). This is because no element can be visited twice yet the DFS algorithm will eventually backtrack and attempt to go in their direction possibly including them in the result. Typically, you might want to filter out all edges with `.where_().node()` condition.

### Paths

Path search (`from().to()`) uses A\* algorithm. Every element (node or edge) has a cost of `1` by default. If it passes all the conditions (the `SearchControl` value `true`) the cost will remain `1` and would be included in the result (if the path it is on would be selected). If it fails any of the conditions (the `SearchControl` value `false`) its cost will be `2`. This means that the algorithm will prefer paths where elements match the conditions rather than the absolutely shortest path (that can be achieved with no conditions). If the search is not to continue beyond certain element (through `beyond()`, `not_beyond()` or `distance()` conditions) its cost will be `0` and the paths it is on will no longer be considered for that search.

### Conditions

<table><tr><td><b>Struct</b></td></tr>
<tr><td>

```rs
pub struct QueryCondition {
    pub logic: QueryConditionLogic,
    pub modifier: QueryConditionModifier,
    pub data: QueryConditionData,
}

pub enum QueryConditionLogic {
    And,
    Or,
}

pub enum QueryConditionModifier {
    None,
    Beyond,
    Not,
    NotBeyond,
}

pub enum QueryConditionData {
    Distance(CountComparison),
    Edge,
    EdgeCount(CountComparison),
    EdgeCountFrom(CountComparison),
    EdgeCountTo(CountComparison),
    Ids(Vec<QueryId>),
    KeyValue { key: DbValue, value: Comparison },
    Keys(Vec<DbValue>),
    Node,
    Where(Vec<QueryCondition>),
}

pub enum CountComparison {
    Equal(u64),
    GreaterThan(u64),
    GreaterThanOrEqual(u64),
    LessThan(u64),
    LessThanOrEqual(u64),
    NotEqual(u64),
}

pub enum Comparison {
    Equal(DbValue),
    GreaterThan(DbValue),
    GreaterThanOrEqual(DbValue),
    LessThan(DbValue),
    LessThanOrEqual(DbValue),
    NotEqual(DbValue),
    Contains(DbValue),
}
```

</td></tr><tr><td><b>Builder</b></td></tr><tr><td>

```rs
//the where_() can be applied to any of the basic search queries after order_by/offset/limit
//not() and not_beyond() can be applied to all conditions including nested where_()
QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query();
QueryBuilder::search().from(1).where_().edge().query();
QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2)).query();
QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1)).query();
QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1)).query();
QueryBuilder::search().from(1).where_().node().query();
QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query();
QueryBuilder::search().from(1).where_().keys(vec!["k1".into(), "k2".into()]).query();
QueryBuilder::search().from(1).where_().not().keys(vec!["k1".into(), "k2".into()]).query();
QueryBuilder::search().from(1).where_().ids([1, 2]).query();
QueryBuilder::search().from(1).where_().beyond().keys(vec!["k".into()]).query();
QueryBuilder::search().from(1).where_().not().ids([1, 2]).query();
QueryBuilder::search().from(1).where_().not_beyond().ids("a").query();
QueryBuilder::search().from(1).where_().node().or().edge().query();
QueryBuilder::search().from(1).where_().node().and().distance(CountComparison::GreaterThanOrEqual(3)).query();
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query();
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(1.into())).end_where().query();
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Contains(vec![1, 2].into())).end_where().query();
```

</td></tr></table>

The currently supported conditions are:

- Where (opens nested list of conditions)
- Edge (if the element is an `edge`)
- Node (if the element is a `node`)
- Distance (if the current distance of the search satisfies the numerical comparison, each graph element away from the start increases the distance, including edges, i.e. second node from start is at distance `2`)
- EdgeCount (if the element is a node and total number of edges (in and out) satisfies the numerical comparison - self-referential edges are counted twice)
- EdgeCountFrom (if the element is a node and total number of outgoing edges satisfies the numerical comparison)
- EdgeCountTo (if the element is a node and total number of incoming edges satisfies the numerical comparison)
- Ids (if the element `id` is in the list)
- KeyValue (if the element's property has the `key` and its value satisfies `value` comparison)
- Keys (if the element has all the `keys` regardless of their values)
- EndWhere (closes nested list of conditions)

All conditions can be further modified as follows:

- Beyond (continues the search only beyond this element)
- Not (reverses the condition result)
- NotBeyond (stops the search beyond this element)

The conditions can be changed with logic operators:

- And (logical `and`)
- Or (logical `or`)

NOTE: The use of `where_` with an underscore as the method name is necessary to avoid conflict with the Rust keyword.

The conditions are applied one at a time to each visited element and chained using logic operators `AND` and `OR`. They can be nested using `where_` and `end_where` (in place of brackets). The condition evaluator supports short-circuiting not evaluating conditions further if the logical outcome cannot change. The condition comparators are type strict meaning that they do not perform type conversions nor coercion (e.g. `Comparison::Equal(1_i64).compare(1_u64)` will evaluate to `false`). Slight exception to this rule is the `Comparison::Contains` as it allows vectorized version of the base type (e.g. `Comparison::Contains(vec!["bc", "ef"]).compare("abcdefg")` will evaluate to `true`).

The condition `Distance` and the condition modifiers `Beyond` and `NotBeyond` are particularly important because they can directly influence the search. The former (`Distance`) can limit the depth of the search and can help with constructing more elaborate queries (or sequence thereof) extracting only fine-grained elements (e.g. nodes whose edges have particular properties or are connected to other nodes with some properties). The latter (`Beyond` and `NotBeyond`) can limit search to only certain areas of an otherwise larger graph. Its most basic usage would be with condition `ids` to flat out stop the search at certain elements or continue only beyond certain elements.

### Truth tables

The following information should help with reasoning about the query conditions. Most of it should be intuitive, but there are some aspects that might not be obvious especially when combining logic operators and condition modifiers. The search is using the following `enum` when evaluating conditions:

```rs
pub enum SearchControl {
    Continue(bool),
    Finish(bool),
    Stop(bool),
}
```

The type controls the search and the boolean value controls if the given element should be included in a search result. The `Stop` will prevent the search expanding beyond current element (stopping the search in that direction). `Finish` will immediately exit the search returning accumulated elements (`ids`) and is only used internally with `offset` and `limit` (NOTE: path search and `order_by` still require complete search regardless of `limit`).

Each condition contributes to the final control result as follows with the starting/default value being always `Continue(true)`:

#### And

| Left           | Right           | Result                  |
| -------------- | --------------- | ----------------------- |
| Continue(left) | Continue(right) | Continue(left && right) |
| Continue(left) | Stop(right)     | Stop(left && right)     |
| Continue(left) | Finish(right)   | Finish(left && right)   |
| Stop(left)     | Stop(right)     | Stop(left && right)     |
| Stop(left)     | Finish(right)   | Finish(left && right)   |
| Finish(left)   | Finish(right)   | Finish(left && right)   |

#### Or

| Left           | Right           | Result                    |
| -------------- | --------------- | ------------------------- |
| Continue(left) | Continue(right) | Continue(left \|\| right) |
| Continue(left) | Stop(right)     | Continue(left \|\| right) |
| Continue(left) | Finish(right)   | Continue(left \|\| right) |
| Stop(left)     | Stop(right)     | Stop(left \|\| right)     |
| Stop(left)     | Finish(right)   | Stop(left \|\| right)     |
| Finish(left)   | Finish(right)   | Finish(left \|\| right)   |

#### Modifiers

Modifiers will change the result of a condition based on the control value (the boolean) as follows:

| Modifier  | TRUE                | FALSE            |
| --------- | ------------------- | ---------------- |
| None      | -                   | -                |
| Beyond    | `&& Continue(true)` | `Stop(true)`     |
| Not       | `!`                 | `!`              |
| NotBeyond | `&& Stop(true)`     | `Continue(true)` |

#### Results

Most conditions result in `Continue(bool)` except for `distance()` and nested `where()` which can also result in `Stop(bool)`:

| Condition   | Continue | Stop |
| ----------- | -------- | ---- |
| Where       | YES      | YES  |
| Edge        | YES      | NO   |
| Node        | YES      | NO   |
| Distance    | YES      | YES  |
| EdgeCount\* | YES      | NO   |
| Ids         | YES      | NO   |
| Key(Value)  | YES      | NO   |
| Keys        | YES      | NO   |

---

For further examples and use cases see the [efficient agdb](/docs/references/efficient-agdb).
