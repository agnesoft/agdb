# Queries

- [Queries](#queries)
- [QueryResult](#queryresult)
- [QueryError](#queryerror)
- [Transactions](#transactions)
- [QueryIds \& QueryId](#queryids--queryid)
- [QueryValues](#queryvalues)
- [Mutable queries](#mutable-queries)
  - [Insert](#insert)
    - [Insert nodes](#insert-nodes)
    - [Insert edges](#insert-edges)
    - [Insert aliases](#insert-aliases)
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
    - [Conditions](#conditions)
    - [Paths](#paths)

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
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).query();
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).query();
QueryBuilder::insert().edges().from(QueryBuilder::search().from("a").where_().node().query()).to(QueryBuilder::search().from("b").where_().node().query()).values_uniform(vec![("k", "v").into(), (1, 10).into()]).query();
```

The `from` and `to` represents list of origins and destinations of the edges to be inserted. As per [`QueryIds`](#queryids--queryid) it can be a list, single value, search query or even a result of another query (e.g. [insert nodes](#insert-nodes)) through the call of convenient `QueryResult::ids()` method. All ids must be `node`s and all must exist in the database otherwise data error will occur. If the `values` is [`QueryValues::Single`](#queryvalues) all edges will be associated with the copy of the same properties. If `values` is [`QueryValues::Multi`](#queryvalues) then the number of edges being inserted must match the provided values otherwise a logic error will occur. By default the `from` and `to` are expected to be of equal length specifying at each index the pair of nodes to connect with an edge. If all-to-all is desired set the `each` flag to `true`. The rule about the `values` [`QueryValues::Multi`](#queryvalues) still applies though so there must be enough values for all nodes resulting from the combination.

The result will contain:

- number of edges inserted
- list of elements inserted with their ids (negative) but without the inserted values

### Insert aliases

```Rust
pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}
```

Builder pattern:

```Rust
QueryBuilder::insert().aliases("a").ids(1).query();
QueryBuilder::insert().aliases("a").ids("b").query(); // alias "b" is replaced  with "a"
QueryBuilder::insert().aliases(vec!["a", "b"]).ids(vec![1, 2]).query();
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
QueryBuilder::insert().values(vec![vec![("k", "v").into(), (1, 10).into()], vec![("k", 2).into()]]).ids(QueryBuilder::search().from("a").query()).query();
QueryBuilder::insert().values_uniform(vec![("k", "v").into(), (1, 10).into()]).ids(vec![1, 2]).query();
QueryBuilder::insert().values_uniform(vec![("k", "v").into(), (1, 10).into()]).ids(QueryBuilder::search().from("a").query()).query();
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
QueryBuilder::remove().ids(QueryBuilder::search().from("a").query()).query();
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
pub struct RemoveValuesQuery(pub SelectValuesQuery);
```

NOTE: See [`SelectValuesQuery`](#select-values) for more details.

Builder pattern:

```Rust
QueryBuilder::remove().values(vec!["k1".into(), "k2".into()]).ids(vec![1, 2]).query();
QueryBuilder::remove().values(vec!["k1".into(), "k2".into()]).ids(QueryBuilder::search().from("a").query()).query();
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
QueryBuilder::select().ids(QueryBuilder::search().from(1).query()).query();
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
QueryBuilder::select().values(vec!["k".into(), "k2".into()]).ids(QueryBuilder::search().from(1).query()).query();
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
QueryBuilder::select().keys().ids(QueryBuilder::search().from(1).query()).query();
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
QueryBuilder::select().key_count().ids(QueryBuilder::search().from(1).query()).query();
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
QueryBuilder::select().aliases().ids(QueryBuilder::search().from(1).query()).query();
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

There is only a single search query that provides the ability to search the graph examining connected elements and their properties. While it is possible to construct the search queries manually, specifying conditions manually in particular can be excessively difficult and therefore **using the builder pattern is recommended**. The default search algorithm is `breadth first` however you can choose to use `depth first`. For path search the `A*` algorithm is used.

```Rust
pub struct SearchQuery {
    pub algorithm: SearchQueryAlgorithm,
    pub origin: QueryId,
    pub destination: QueryId,
    pub limit: u64,
    pub offset: u64,
    pub order_by: Vec<DbKeyOrder>,
    pub conditions: Vec<QueryCondition>,
}

pub enum SearchQueryAlgorithm {
    BreadthFirst,
    DepthFirst,
}

pub enum DbKeyOrder {
    Asc(DbKey),
    Desc(DbKey),
}
```

Builder pattern:

```Rust
QueryBuilder::search().from("a").query();
QueryBuilder::search().to(1).query(); //reverse search
QueryBuilder::search().from("a").to("b").query(); //path search, A*

QueryBuilder::search().breadth_first().from("a").query(); //breadth first is the default and can be omitted however
QueryBuilder::search().depth_first().from("a").query();

//limit, offset and order_by can be applied similarly to all the search variants
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("age".into()), DbKeyOrder::Asc("name".into())]).query()
QueryBuilder::search().from(1).offset(10).query();
QueryBuilder::search().from(1).limit(5).query();
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).query();
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).limit(5).query();
QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Desc("k".into())]).offset(10).limit(5).query();
QueryBuilder::search().from(1).offset(10).limit(5).query();
```

The search query is made up of the `origin` and `destination` of the search and the algorithm. Specifying only `origin` (from) will result in a search along `from->to` edges. Specifying only `destination` (to) will result in the reverse search along the `to<-from` edges. When both `origin` and `destination` are specified the search algorithm becomes a path search and the algorithm used will be `A*`. Optionally you can specify a `limit` (0 = unlimited) and `offset` (0 = no offset) to the returned list of graph element ids. If specified (!= 0) the `origin` and the `destination` must exist in the database, otherwise an error will be returned. The elements can be optionally ordered with `order_by` list of keys allowing ascending/descending ordering based on multiple properties.

Finally the list of `conditions` that each examined graph element must satisfy to be included in the result (and subjected to the `limit` and `offset`).

**NOTE:** When both `origin` and `destination` are specified and the algorithm is switched to the `A*` the `limit` and `offset` are applied differently. In regular (open-ended) search the search will end when the `limit` is reached but with the path search (A\*) the `destination` must be reached first before they are applied.

### Conditions

The currently supported conditions are:

- Where (opens nested list of conditions)
- Edge (if the element is an `edge`)
- Node (if the element is a `node`)
- Distance (if the current distance of the search satisfies the numerical comparison, each graph element away from the start increases the distance, including edges, i.e. second node from start is at distance `2`)
- EdgeCount (if the element is a node and total number of edges (in and out) satisfies the numerical comparison - self-referential edges are counted twice)
- EdgeCountFrom (if the element is a node and total number of outgoing edges satisfies the numerical comparison)
- EdgeCountTo (if the element is a node and total number of incoming edges satisfies the numerical comparison)
- Ids (if the element id is in the list)
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

```Rust
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
    KeyValue { key: DbKey, value: Comparison },
    Keys(Vec<DbKey>),
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
}
```

Builder pattern:

```Rust
//the where_() can be applied to any of the basic search queries after order_by/offset/limit
//not() and not_beyond() can be applied to all conditions including nested where_()
QueryBuilder::search().from(1).where_().distance(CountComparison::LessThan(3)).query();
QueryBuilder::search().from(1).where_().edge().query();
QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(2))().query();
QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1))().query();
QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::NotEqual(1))().query();
QueryBuilder::search().from(1).where_().node().query();
QueryBuilder::search().from(1).where_().key("k").value(Comparison::Equal(1.into())).query();
QueryBuilder::search().from(1).where_().keys(vec!["k1".into(), "k2".into()]).query();
QueryBuilder::search().from(1).where_().not().keys(vec!["k1".into(), "k2".into()]).query();
QueryBuilder::search().from(1).where_().ids(vec![1, 2]).query();
QueryBuilder::search().from(1).where_().beyond().keys(vec!["k"]).query();
QueryBuilder::search().from(1).where_().not().ids(vec![1, 2]).query();
QueryBuilder::search().from(1).where_().not_beyond().ids("a").query();
QueryBuilder::search().from(1).where_().node().or().edge().query();
QueryBuilder::search().from(1).where_().node().and().distance().query(CountComparison::GreaterThanOrEqual(3)).query();
QueryBuilder::search().from(1).where_().node().or().where_().edge().and().key("k").value(Comparison::Equal(1.into())).end_where().query();
```

NOTE: The use of `where_` with an underscore as the method name is necessary to avoid conflict with the Rust keyword.

The conditions are applied one at a time to each visited element and chained using logic operators `AND` and `OR`. They can be nested using `where_` and `end_where` (in place of brackets). The condition evaluator supports short-circuiting not evaluating conditions further if the logical outcome cannot change.

The condition `Distance` and the condition modifiers `Beyond` and `NotBeyond` are particularly important because they can directly influence the search. The former (`Distance`) can limit the depth of the search and can help with constructing more elaborate queries (or sequence thereof) extracting only fine grained elements (e.g. nodes whose edges have particular properties or are connected to other nodes with some properties). The latter (`Beyond` and `NotBeyond`) can limit search to only certain areas of an otherwise larger graph. Its most basic usage would be with condition `ids` to flat out stop the search at certain elements or continue only beyond certain elements.

### Paths

Path search (`from().to()`) uses A\* algorithm. Every element (node or edge) has a cost of `1` by default. If it passes all the conditions the cost will remain `1` and would be included in the result (if the path it is on would be selected). If it fails any of the conditions its cost will be `2`. This means that the algorithm will prefer paths where elements match the conditions rather than the absolutely shortest path (that can be achieved with no conditions). If the search is not to continue beyond certain element (through `beyond()` or `not_beyond()` conditions) its cost will be `0` and the paths it is on will no longer be considered for that search.

---

For further examples and use cases see the [efficient agdb](docs/efficient_agdb.md).
