---
name: agdb-query
description: Build, review, and explain agdb queries accurately. Use this skill when writing QueryBuilder chains, converting between search/select/insert/remove, reasoning about where_ conditions, or debugging query logic and result semantics in agdb.
argument-hint: "[build|review|debug] [query intent in plain words]"
---

# agdb Query Skill

Use this skill when working with agdb query construction and behavior, especially:

- QueryBuilder chains in Rust
- `search` + `where_` conditions and modifiers
- Converting query intent into correct `insert` / `remove` / `select` / `search`
- Understanding `QueryResult` and `DbType` mapping
- Debugging query logic errors vs data errors
- Value amendment operations (`insert().amend()`, `remove().amend()`, `insert().amend_or()`, etc.)

Primary sources in this repository:

- `agdb/src/query.rs`
- `agdb/src/query_builder.rs`
- `agdb/src/query_builder/*`
- `agdb/src/query/*`
- `agdb_web/content/docs/03.references/01.queries.md`

## Mental model

1. `QueryBuilder` builds typed query objects.
2. Query objects are executed by `Db::exec` (immutable) or `Db::exec_mut` (mutable).
3. Every query returns `Result<QueryResult, DbError>`.
4. `QueryResult` contains:
   - `result`: numeric aggregate (count or similar)
   - `elements`: element payloads (ids and/or values depending on query, always present, can be empty)
5. `QueryResult::ids()` extracts just the `DbId` of every element — useful for chaining.
6. `QueryResult` implements `TryInto<Vec<T>>` and `TryInto<T>` for `DbType`/`DbElement` types.

## Execution rules

- Use `exec` for immutable queries: `select`, `search`.
- Use `exec_mut` for mutable queries: `insert`, `remove`.
- All query execution is transactional under the hood.
- Explicit transactions (`transaction` / `transaction_mut`) are for grouping multiple queries.
- In explicit transactions, the closure receives a `Transaction`/`TransactionMut` with the same `exec`/`exec_mut` methods.

## QueryBuilder cheat sheet

### Immutable (use with `exec`)

```rs
// Select all values by id
QueryBuilder::select().ids(1).query();
// Select specific keys
QueryBuilder::select().values(["k".into()]).ids([1, 2]).query();
// Select keys only (no values)
QueryBuilder::select().keys().ids([1, 2]).query();
// Select key count
QueryBuilder::select().key_count().ids([1, 2]).query();
// Select edge count (from/to/both)
QueryBuilder::select().edge_count_from().ids(1).query();
QueryBuilder::select().edge_count_to().ids(1).query();
QueryBuilder::select().edge_count().ids(1).query();
// Select aliases
QueryBuilder::select().aliases().ids(1).query();
QueryBuilder::select().aliases().query(); // all aliases
// Select indexes
QueryBuilder::select().indexes().query();
// Select node count
QueryBuilder::select().node_count().query();
// Search - BFS (default) from alias
QueryBuilder::search().from("alias").query();
// Search - path search (A*)
QueryBuilder::search().from(1).to(2).query();
// Search - index lookup (fastest for single-key equality)
QueryBuilder::search().index("username").value("alice").query();
// Search - full element scan
QueryBuilder::search().elements().query();
// Nested: select values via search result
QueryBuilder::select().search().from("alias").where_().node().query();
// Nested: typed select (single result, limit=1)
QueryBuilder::select().element::<User>().search().from("alias").where_().key("name").value("Alice").query();
// Nested: typed select (all matches)
QueryBuilder::select().elements::<User>().search().from("alias").where_().node().query();
```

### Mutable (use with `exec_mut`)

```rs
// Insert nodes
QueryBuilder::insert().nodes().count(1).query();
QueryBuilder::insert().nodes().aliases(["root"]).query();
QueryBuilder::insert().nodes().values(&typed_vec).query();
// Upsert (insert or update): pass existing ids
QueryBuilder::insert().nodes().ids(existing_ids).values(&typed_vec).query();
// Insert edges
QueryBuilder::insert().edges().from(1).to(2).query();
QueryBuilder::insert().edges().from("alias_a").to([3, 4, 5]).query();
QueryBuilder::insert().edges().from(1).to(2).values([[("weight", 10).into()]]).query();
// Insert/update aliases
QueryBuilder::insert().aliases(["name"]).ids(1).query();
// Insert/update values
QueryBuilder::insert().values([vec![("k", 1).into()]]).ids(1).query();
QueryBuilder::insert().values_uniform([("k", 1).into()]).ids(1).query();
// Insert single typed element (DbElement)
QueryBuilder::insert().element(typed_element).query();
// Insert multiple typed elements (DbElement)
QueryBuilder::insert().elements(&typed_elements).query();
// Create index
QueryBuilder::insert().index("key").query();
// Remove elements
QueryBuilder::remove().ids([1, 2]).query();
// Remove aliases (not the elements)
QueryBuilder::remove().aliases("name").query();
// Remove specific properties
QueryBuilder::remove().values("key").ids(1).query();
// Remove index
QueryBuilder::remove().index("key").query();
```

### Value amendments

Amendment operations modify existing values in place rather than replacing them:

```rs
// Add to numeric value (saturating), append to string/vec
QueryBuilder::insert().amend([vec![("score", 10).into()]]).ids(1).query();
// Same value applied uniformly to multiple elements
QueryBuilder::insert().amend_uniform([("score", 10).into()]).ids([1, 2]).query();
// Subtract from numeric value, remove first occurrence from string/vec
QueryBuilder::remove().amend([vec![("tags", "old_tag").into()]]).ids(1).query();
// Bitwise operations on integer/byte values
QueryBuilder::insert().amend_or([vec![("flags", 0x04_u64).into()]]).ids(1).query();
QueryBuilder::insert().amend_and([vec![("mask", 0xFF_u64).into()]]).ids(1).query();
QueryBuilder::insert().amend_xor([vec![("toggle", 0x01_u64).into()]]).ids(1).query();
// Uniform variants: amend_or_uniform, amend_and_uniform, amend_xor_uniform
```

**`amend` (add) behavior by type:**

| Type         | Behavior                                     |
| ------------ | -------------------------------------------- |
| `I64`, `U64` | Saturating addition                          |
| `F64`        | Floating-point addition                      |
| `String`     | Concatenation                                |
| `Bytes`      | Byte vector extension                        |
| `Vec*`       | Extend with elements (or push single scalar) |

**`remove().amend()` behavior by type:**

| Type         | Behavior                               |
| ------------ | -------------------------------------- |
| `I64`, `U64` | Saturating subtraction                 |
| `F64`        | Floating-point subtraction             |
| `String`     | Remove first occurrence of substring   |
| `Bytes`      | Not supported (returns error)          |
| `Vec*`       | Remove first matching element per item |

## Builder behavior that is easy to miss

- `QueryBuilder::search()` defaults to breadth-first search.
- `QueryBuilder::select().search()` is shorthand for selecting values via a nested search query.
- `QueryBuilder::insert().values(...).search()` is shorthand for using search result ids as insert targets.
- `QueryBuilder::insert().nodes().ids(...)` enables insert-or-update semantics:
  - empty ids result: insert
  - non-empty ids result: update existing nodes
- `QueryBuilder::insert().edges().ids(...)` does the same for edges (updates values of found edge ids).
- `QueryBuilder::select().element::<T>()` sets `limit = 1` and uses `T::db_keys()`.
- If `T::db_element_id()` is `Some`, typed `select/search` adds a `db_element_id` filter automatically.
- `QueryBuilder::insert().element(x)` and `elements(&xs)` only work with types implementing `DbElement` (not just `DbType`).
- `search().depth_first()` changes traversal to DFS — more efficient when searching for a single item.

## Conditions (`where_`) guidance

Condition chains compile into `QueryCondition` items with:

- logic: `And` or `Or`
- modifier: `None`, `Not`, `Beyond`, `NotBeyond`
- data: node/edge/ids/key-value/distance/etc.

### Most useful primitives

```rs
.where_().node()
.where_().edge()
.where_().key("k").value(1)
.where_().keys(["k1".into(), "k2".into()])
.where_().distance_less_than_or_equal(2)
.where_().ids([1, 2, 3])
```

### Logic operators

```rs
.where_().node().or().edge()
.where_().node().and().key("k").value("v")
.where_().node().or().where_().edge().and().key("k").value(1).end_where()
```

**Important**: `or()` passes if **either side** evaluates to true. `and()` passes only if **both sides** evaluate to true.

### Modifiers

- `not()`: negates selection result of the next condition.
- `beyond()`: controls traversal only; continue past an element only if condition passes.
- `not_beyond()`: controls traversal only; stop past an element if condition passes.

Important: `beyond()` / `not_beyond()` do not directly select/reject elements by themselves. Pair them with normal selection conditions when needed.

### Convenience helpers

**Distance shorthands** — avoid manually constructing `CountComparison`:

```rs
.where_().distance_less_than(3)              // same as .distance(CountComparison::LessThan(3))
.where_().distance_less_than_or_equal(3)
.where_().distance_greater_than(1)
.where_().distance_greater_than_or_equal(1)
.where_().distance_not_equal(0)
.where_().neighbor()                         // shorthand for distance == 2
```

**Value comparison shorthands** — avoid manually constructing `Comparison`:

```rs
.where_().key("age").greater_than(18)        // same as .value(Comparison::GreaterThan(18.into()))
.where_().key("age").greater_than_or_equal(18)
.where_().key("age").less_than(65)
.where_().key("age").less_than_or_equal(65)
.where_().key("name").not_equal("admin")
.where_().key("name").contains("alice")      // substring/element match
.where_().key("name").starts_with("user_")
.where_().key("name").ends_with("_admin")
.where_().key("role").any(["admin", "mod"])   // matches if value is any of the list
.where_().key("name").regex("^user_\\d+$")   // regex match on string values
```

**Edge count shorthands** — avoid manually constructing `CountComparison`:

```rs
.where_().edge_count_from_greater_than(0)    // has outgoing edges
.where_().edge_count_to_less_than(5)         // fewer than 5 incoming edges
// also: edge_count_*, edge_count_from_*, edge_count_to_* with _less_than, _greater_than, _not_equal, etc.
```

**Typed element filter**:

- `where_().element::<T>()`:
  - uses `db_element_id` filter if available
  - otherwise falls back to keys-based condition

### Known caveat

- `where_().ids(...)` does not support nested search ids in this condition context; a search passed there is ignored.

## Search semantics

- `from(x)`: forward traversal (starting from x)
- `to(x)`: reverse traversal following incoming edges (starting from x)
- `from(x).to(y)`: path search (A*) going from x to y
- `to(y).from(x)`: reverse path search — follows incoming edges, can be significantly faster when the destination has fewer incoming edges than the origin has outgoing
- `elements()`: full element scan (can be expensive)
- `index("key").value(v)`: indexed lookup — supports `limit`, `offset`, `order_by`, and `where_()` conditions just like graph searches

Search supports breadth-first (BFS, default) and depth-first (DFS, `depth_first()`) traversal order, can be selected in the builder query (`QueryBuilder::search().depth_first()...`).
This changes order in which the elements are visited and therefore the order of results and how `beyond/not_beyond` conditions are applied.

Use `limit` and `offset` for large traversals. For `elements()` queries, always consider `limit` first.

You can also order results with `order_by`:

```rs
QueryBuilder::search().from("root").order_by([DbKeyOrder::Asc("name".into())]).query();
```

## Query type selection rubric

- Need ids only by traversal/filtering: `search`
- Need values/properties: `select` (optionally with nested `search`)
- Need create/update: `insert`
- Need delete aliases/elements/index/values: `remove`

## Common pitfalls and corrections

1. Pitfall: using `exec` with mutable query.
   - Fix: use `exec_mut`.

2. Pitfall: `insert().values(Multi)` count does not match target ids.
   - Fix: align lengths or switch to `values_uniform`.

3. Pitfall: expecting `search` to return full values.
   - Fix: use `select().search()...` when values are needed.

4. Pitfall: forgetting `.query()` at end of builder chain.
   - Fix: terminate chain with `.query()` before execution.

5. Pitfall: mixing traversal control with selection logic.
   - Fix: keep `beyond/not_beyond` for traversal and add explicit node/edge/key conditions for selection.

6. Pitfall: using `element()` (singular) when expecting multiple results.
   - Fix: use `elements::<T>()` (plural) for multiple matches. `element::<T>()` sets `limit = 1`.

7. Pitfall: constructing queries as raw JSON via curl or HTTP client.
   - Fix: always use `QueryBuilder` in Rust. The query serialization format has deeply nested enums that are not designed for manual construction. Use the `agdb_api` client SDK instead.

8. Pitfall: expecting `amend` on incompatible types to silently no-op.
   - Fix: amendment operations return `DbError` when types don't match (e.g. adding I64 to String). Ensure the stored value type matches the amendment value type.

## Preferred answer style when AI writes queries

When generating query code:

1. State whether query is immutable or mutable.
2. Show the exact builder chain.
3. Explain expected `QueryResult.result` meaning.
4. Mention likely error cases (missing ids, mismatched values length, invalid aliases).
5. If using typed mapping (`DbType`), call out required keys and optional `db_id` behavior.

## Minimal examples

### Find user nodes by indexed username and select typed result

```rs
let q = QueryBuilder::select()
    .elements::<User>()
    .search()
    .index("username")
    .value("alice")
    .query();
let result = db.exec(q)?;
let users: Vec<User> = result.try_into()?;
```

### Update existing node values by id

```rs
let q = QueryBuilder::insert()
    .values_uniform([("active", 1).into()])
    .ids(42)
    .query();
db.exec_mut(q)?;
```

### Traverse only through admin-tagged nodes

```rs
let q = QueryBuilder::search()
    .from("root")
    .where_()
    .beyond()
    .key("role")
    .value("admin")
    .query();
let ids = db.exec(q)?.ids();
```

### Insert and link nodes in one transaction

```rs
db.transaction_mut(|t| {
    let root = t.exec_mut(
        QueryBuilder::insert().nodes().aliases(["users"]).query()
    )?;
    let users = t.exec_mut(
        QueryBuilder::insert().nodes().values(&user_data).query()
    )?;
    t.exec_mut(
        QueryBuilder::insert().edges().from("users").to(users).query()
    )
})?;
```

### Increment a counter value

```rs
db.exec_mut(
    QueryBuilder::insert()
        .amend([vec![("visit_count", 1_i64).into()]])
        .ids("page_home")
        .query()
)?;
```

### Emulate a relational join (DFS + edge/node interleaving)

```rs
let result = db.exec(
    QueryBuilder::select()
        .search()
        .depth_first()
        .from("user")
        .where_()
        .keys("role")
        .or()
        .keys("name")
        .query(),
)?;
// DFS guarantees: edge (id < 0) followed by its target node (id > 0)
for element in result.elements {
    if element.id.0 < 0 {
        // edge — extract relationship properties
    } else {
        // node — extract entity properties
    }
}
```

### Full agentic workflow: connect, create, query via agdb_api

```rs
use agdb::{DbType, QueryBuilder};
use agdb_api::{AgdbApi, ReqwestClient, DbKind};

#[derive(Debug, DbType)]
struct User {
    username: String,
    email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = AgdbApi::new(ReqwestClient::new(), "localhost:3000");
    client.user_login("admin", "admin").await?;
    client.db_add("admin", "my_db", DbKind::Mapped).await?;

    // Create root + users + edges in one atomic call
    let users = vec![
        User { username: "alice".into(), email: "a@b.com".into() },
        User { username: "bob".into(), email: "b@b.com".into() },
    ];
    let results = client.db_exec_mut("admin", "my_db", &[
        QueryBuilder::insert().nodes().aliases(["users"]).query().into(),
        QueryBuilder::insert().nodes().values(&users).query().into(),
    ]).await?.1;

    client.db_exec_mut("admin", "my_db", &[
        QueryBuilder::insert().edges().from("users").to(results[1].ids()).query().into(),
        QueryBuilder::insert().index("username").query().into(),
    ]).await?;

    // Query back
    let (_, results) = client.db_exec("admin", "my_db", &[
        QueryBuilder::select()
            .elements::<User>()
            .search()
            .index("username")
            .value("alice")
            .query()
            .into(),
    ]).await?;
    let found: Vec<User> = results[0].clone().try_into()?;
    println!("{found:?}");

    Ok(())
}
```

## Validation checklist for AI before finalizing query code

- Correct top-level builder (`insert/remove/select/search`)
- Correct execution method (`exec` vs `exec_mut`)
- `.query()` present
- `.into()` present when batching queries in a `Vec`
- `Multi` vs `Single` values shape correct
- Conditions use intended logic and modifiers
- Any typed conversion (`try_into`) matches selected keys
- Amendment operations match stored value types
- Never construct raw JSON — always use `QueryBuilder`
