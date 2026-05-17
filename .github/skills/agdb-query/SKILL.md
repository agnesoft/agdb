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

## Execution rules

- Use `exec` for immutable queries: `select`, `search`.
- Use `exec_mut` for mutable queries: `insert`, `remove`.
- All query execution is transactional under the hood.
- Explicit transactions (`transaction` / `transaction_mut`) are for grouping multiple queries.

## QueryBuilder cheat sheet

### Immutable

```rs
QueryBuilder::select().ids(1).query();
QueryBuilder::select().values(["k".into()]).ids([1, 2]).query();
QueryBuilder::search().from("alias").query();
QueryBuilder::search().from(1).to(2).query(); // path search (A*)
```

### Mutable

```rs
QueryBuilder::insert().nodes().count(1).query();
QueryBuilder::insert().edges().from(1).to(2).query();
QueryBuilder::insert().values_uniform([("k", 1).into()]).ids(1).query();
QueryBuilder::remove().ids([1, 2]).query();
```

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
.where_().distance(CountComparison::LessThanOrEqual(2))
```

### Logic operators

```rs
.where_().node().or().edge()
.where_().node().and().key("k").value("v")
.where_().node().or().where_().edge().and().key("k").value(1).end_where()
```

### Modifiers

- `not()`: negates selection result of the next condition.
- `beyond()`: controls traversal only; continue past an element only if condition passes.
- `not_beyond()`: controls traversal only; stop past an element if condition passes.

Important: `beyond()` / `not_beyond()` do not directly select/reject elements by themselves. Pair them with normal selection conditions when needed.

### Convenience helpers

- `neighbor()` is shorthand for distance `== 2` from origin node in graph traversal terms.
- `where_().element::<T>()`:
  - uses `db_element_id` filter if available
  - otherwise falls back to keys-based condition

### Known caveat

- `where_().ids(...)` does not support nested search ids in this condition context; a search passed there is ignored.

## Search semantics

- `from(x)`: forward traversal (starting from x)
- `to(x)`: reverse traversal following incoming edges (starting from x)
- `from(x).to(y)`: path search (A*) going from x to y
- `elements()`: full element scan (can be expensive)
- `index("key").value(v)`: indexed lookup path

Search supports breadth-first (BFS, default) and depth-first (DPS, `depth_first()`) traversal order, can be selected in the builder query (`QueryBuilder::search().depth_first()...`).
This changes order in which the elements are visited and therefore the order of results and how `beyond/not_beyond` conditions are applied.

Use `limit` and `offset` for large traversals. For `elements()` queries, always consider `limit` first.

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

## Validation checklist for AI before finalizing query code

- Correct top-level builder (`insert/remove/select/search`)
- Correct execution method (`exec` vs `exec_mut`)
- `query()` present
- `Multi` vs `Single` values shape correct
- Conditions use intended logic and modifiers
- Any typed conversion (`try_into`) matches selected keys
