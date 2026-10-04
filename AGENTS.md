# AGENTS.md

The Agnesoft Graph Database (aka agdb) is an embedded/server graph database written in Rust. It stores data as nodes and edges with key-value properties, supports ACID transactions via a write-ahead log (WAL), and offers multiple storage backends. The main components of this repository are:

| Crate         | Purpose                                                                                                         |
| ------------- | --------------------------------------------------------------------------------------------------------------- |
| `agdb`        | Core database engine — query builder, storage backends, transactions, derive macros                             |
| `agdb_derive` | Proc-macro crate powering `#[derive(DbType)]`, `DbElement`, `DbValue`, `DbSerialize`, `DbTypeMarker`, `TypeDef` |
| `agdb_server` | HTTP server wrapping `agdb` with REST API, auth, multi-tenancy, backup/restore, and optional Raft cluster       |
| `agdb_api`    | Rust client SDK for `agdb_server` (also ships shared request/response types used by all language clients)       |

API client packages are available for Rust, TypeScript, and PHP. The OpenAPI specification is located at `agdb_server/openapi.json`.

# Setup

- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install pnpm: `npm i -g pnpm`. This assumes `npm` is installed & available.
- Install PHP and Composer. Only needed for PHP API client. Use the `php-composer-setup` skill in `.github/skills/php-composer-setup/SKILL.md`.
- Install playwright: `pnpm exec playwright install`. Only for TypeScript functional tests.

# Available commands

- OpenAPI, API refresh and version bump: `cargo run -r -p agdb_ci`. Run when agdb_server/src/api.rs, agdb_server/openapi.json or Version file changes.

## Rust

- Build package: `cargo build -r --all-features -p <package>`
- Build all: `cargo build -r --all-features`
- Format: `cargo fmt`
- Test package: `cargo test -r --all-features -p <package>`
- Test package with coverage: `cargo llvm-cov -p <package> --show-missing-lines`
- Test all: `cargo test -r --all-features`
- Test debug: `cargo test --all-features -p <package>`
- Lint package: `cargo clippy --all-features -p <package>`
- Lint all: `cargo clippy --all-features`

## TypeScript

- Install dependencies: `pnpm i --frozen-lockfile`
- Update dependencies: `pnpm i`
- Build: `pnpm run build --filter <package>`
- Format: `pnpm run format --filter <package>`
- Test: `pnpm run test --filter <package>`
- Run functional tests: `pnpm run test:functional --filter <package>`
- Lint: `pnpm run lint --filter <package>`

## PHP

- Test with coverage: `cd agdb_api/php/ && ./ci.sh coverage`
- Lint: `cd agdb_api/php/ && ./ci.sh analyse`
- Format: `cd agdb_api/php/ && ./ci.sh format`

# Available packages

The packages are curated lists of main packages. They contain exact names of packages directly usable in the available commands.

## Rust

- agdb
- agdb_benchmark
- agdb_ci
- agdb_derive
- agdb_server
- agdb_api
- examples_app_db
- examples_indexes
- examples_joins
- examples_schema_migration
- examples_server_client
- examples_user_types

## TypeScript

- agdb_studio
- @agnesoft/agdb_api
- agdb_web
- examples_server_client

# Core concepts

## Storage backends

The database engine (`DbImpl<Store>`) is generic over storage. Four type aliases are provided:

| Alias      | Storage                   | Use case                                            |
| ---------- | ------------------------- | --------------------------------------------------- |
| `Db`       | `FileStorageMemoryMapped` | Production — memory-mapped file, fastest reads      |
| `DbFile`   | `FileStorage`             | Lower memory footprint, minimum file I/O buffering  |
| `DbMemory` | `MemoryStorage`           | Tests, ephemeral data — no disk I/O                 |
| `DbAny`    | `AnyStorage`              | Runtime-selectable (file, memory-mapped, or memory) |

Each alias also has a `*Transaction` and `*TransactionMut` variant for use inside explicit transaction closures.

## Query execution model

1. Build a query with `QueryBuilder` (typed builder pattern).
2. Execute with `db.exec(query)` (immutable) or `db.exec_mut(query)` (mutable).
3. Every query returns `Result<QueryResult, DbError>`.
4. `QueryResult` contains `result` (numeric aggregate) and `elements` (ids + values).
5. Group multiple queries atomically with `db.transaction(|t| ...)` / `db.transaction_mut(|t| ...)`.

## Derive macros

| Macro          | Purpose                                                                      |
| -------------- | ---------------------------------------------------------------------------- |
| `DbType`       | Struct ↔ key-value property mapping; enables `.values(&structs)` in queries  |
| `DbElement`    | Like `DbType` but adds a `db_element_id` discriminator for overlapping types |
| `DbValue`      | `TryFrom<DbValue>` + `Into<DbValue>` for custom scalar types                 |
| `DbSerialize`  | Binary serialization for storage (needed by `DbValue` types)                 |
| `DbTypeMarker` | Marker trait allowing a type to be used inside `Vec<T>` in `DbType` structs  |
| `TypeDef`      | OpenAPI schema generation for types                                          |

### DbType field attributes

- `#[agdb(skip)]` — field is not stored in the database
- `#[agdb(rename = "name")]` — store under a different key name
- `#[agdb(flatten)]` — inline a nested struct's fields into the parent

## Query types

### Immutable (use with `exec`)

| Builder                                | Query type              | Returns                                                                                     |
| -------------------------------------- | ----------------------- | ------------------------------------------------------------------------------------------- |
| `select().ids(...)`                    | `SelectValuesQuery`     | All key-value properties of elements                                                        |
| `select().values([...]).ids(...)`      | `SelectValuesQuery`     | Specific keys from elements                                                                 |
| `select().keys().ids(...)`             | `SelectKeysQuery`       | Only key names from elements                                                                |
| `select().key_count().ids(...)`        | `SelectKeyCountQuery`   | Number of keys per element                                                                  |
| `select().aliases().ids(...)`          | `SelectAliasesQuery`    | Aliases of given elements                                                                   |
| `select().aliases()`                   | `SelectAllAliasesQuery` | All aliases in the database                                                                 |
| `select().edge_count...ids(...)`       | `SelectEdgeCountQuery`  | Edge counts (from/to/both)                                                                  |
| `select().node_count()`                | `SelectNodeCountQuery`  | Total node count                                                                            |
| `select().indexes()`                   | `SelectIndexesQuery`    | All existing indexes                                                                        |
| `search().from(x)`                     | `SearchQuery`           | BFS from x (forward traversal)                                                              |
| `search().to(x)`                       | `SearchQuery`           | BFS from x (reverse traversal)                                                              |
| `search().from(x).to(y)`               | `SearchQuery`           | A* path search from x to y                                                                  |
| `search().to(y).from(x)`               | `SearchQuery`           | Reverse A* path (follows incoming edges — faster when destination has fewer incoming edges) |
| `search().elements()`                  | `SearchQuery`           | Full element scan                                                                           |
| `search().index("key").value(v)`       | `SearchQuery`           | Indexed lookup                                                                              |
| `select().search()...`                 | Nested                  | Select values via embedded search                                                           |
| `select().element::<T>().search()...`  | Nested typed            | Select typed values (limit=1, auto keys)                                                    |
| `select().elements::<T>().search()...` | Nested typed            | Select typed values (all matches)                                                           |

### Mutable (use with `exec_mut`)

| Builder                                   | Query type           | Returns                          |
| ----------------------------------------- | -------------------- | -------------------------------- |
| `insert().nodes().count(n)`               | `InsertNodesQuery`   | Insert n empty nodes             |
| `insert().nodes().aliases([...])`         | `InsertNodesQuery`   | Insert aliased nodes             |
| `insert().nodes().values(&data)`          | `InsertNodesQuery`   | Insert nodes with values         |
| `insert().nodes().ids(...).values(...)`   | `InsertNodesQuery`   | Upsert: insert or update         |
| `insert().edges().from(x).to(y)`          | `InsertEdgesQuery`   | Insert edges                     |
| `insert().edges().ids(...).values(...)`   | `InsertEdgesQuery`   | Update existing edges            |
| `insert().aliases("a").ids(1)`            | `InsertAliasesQuery` | Set alias on element             |
| `insert().values(...).ids(...)`           | `InsertValuesQuery`  | Add/update properties            |
| `insert().values_uniform([...]).ids(...)` | `InsertValuesQuery`  | Same values to multiple elements |
| `insert().element(typed)`                 | `InsertNodesQuery`   | Insert single DbElement node     |
| `insert().elements(&typed_vec)`           | `InsertNodesQuery`   | Insert/upsert DbElement nodes    |
| `insert().index("key")`                   | `InsertIndexQuery`   | Create a value index             |
| `remove().ids(...)`                       | `RemoveQuery`        | Delete elements                  |
| `remove().aliases("a")`                   | `RemoveAliasesQuery` | Remove aliases (not elements)    |
| `remove().values("key").ids(...)`         | `RemoveValuesQuery`  | Remove specific properties       |
| `remove().index("key")`                   | `RemoveIndexQuery`   | Drop an index                    |

### Value amendment (insert with modify semantics)

`InsertValuesQuery` supports amendment operations that modify existing values in place:

```rs
// Add 10 to the current "score" value (saturating for integers)
QueryBuilder::insert().amend([vec![("score", 10).into()]]).ids(1).query();
// Same value applied uniformly to multiple elements
QueryBuilder::insert().amend_uniform([("score", 10).into()]).ids([1, 2]).query();
// Subtract / remove first occurrence from a list value
QueryBuilder::remove().amend([vec![("tags", "old").into()]]).ids(1).query();
// Bitwise operations on integer/byte values (OR, AND, XOR)
QueryBuilder::insert().amend_or([vec![("flags", 0x04).into()]]).ids(1).query();
QueryBuilder::insert().amend_and([vec![("mask", 0xFF_u64).into()]]).ids(1).query();
QueryBuilder::insert().amend_xor([vec![("toggle", 0x01_u64).into()]]).ids(1).query();
// Uniform variants: amend_or_uniform, amend_and_uniform, amend_xor_uniform
```

### Search conditions (`where_`)

Condition chains filter elements during search:

```rs
.where_().node()                              // only nodes
.where_().edge()                              // only edges
.where_().key("k").value(1)                   // key equals value
.where_().keys(["k1".into(), "k2".into()])    // element has all listed keys
.where_().distance(CountComparison::Equal(2)) // graph distance from origin
.where_().neighbor()                          // shorthand for distance == 2

// Value comparison shorthands (avoid manual Comparison:: construction)
.where_().key("age").greater_than(18)         // Comparison::GreaterThan
.where_().key("age").less_than(65)            // Comparison::LessThan
.where_().key("name").contains("alice")       // Comparison::Contains (substring/element match)
.where_().key("name").starts_with("user_")    // Comparison::StartsWith
.where_().key("role").any(["admin", "mod"])    // Comparison::Any (matches any of the list)
.where_().key("name").regex("^user_\\d+$")    // Comparison::Regex

// Distance shorthands (avoid manual CountComparison:: construction)
.where_().distance_less_than(3)               // CountComparison::LessThan(3)
.where_().distance_greater_than(1)            // CountComparison::GreaterThan(1)

// Edge count shorthands
.where_().edge_count_from_greater_than(0)     // has outgoing edges
.where_().edge_count_to_less_than(5)          // fewer than 5 incoming edges

// Logic
.where_().node().or().edge()
.where_().node().and().key("k").value("v")
.where_().not().key("k").value("v")           // negation

// Nested groups
.where_().node().or().where_().edge().and().key("k").value(1).end_where()

// Traversal control (does NOT select, only controls which branches to explore)
.where_().beyond().key("role").value("admin")
.where_().not_beyond().distance(CountComparison::GreaterThan(3))
```

## Server configuration

The server reads a YAML-like config file (default: `agdb_server.yaml`). On first run it creates a default config.

| Key                            | Default                 | Description                                                                  |
| ------------------------------ | ----------------------- | ---------------------------------------------------------------------------- |
| `bind`                         | `:::3000`               | Socket bind address                                                          |
| `address`                      | `http://localhost:3000` | Public address for cluster node discovery                                    |
| `basepath`                     | (empty)                 | URL path prefix, e.g. `/api`                                                 |
| `static_roots`                 | `[]`                    | Directories to serve as static files                                         |
| `admin`                        | `admin`                 | Admin username                                                               |
| `log_level`                    | `info`                  | Logging level (`trace`, `debug`, `info`, `warn`, `error`, `off`)             |
| `log_body_limit`               | `10240`                 | Max bytes of request/response body to log                                    |
| `request_body_limit`           | `10485760`              | Max request body size (bytes)                                                |
| `data_dir`                     | `agdb_server_data`      | Directory for databases and server state                                     |
| `pepper_path`                  | (empty)                 | Path to 16-byte pepper file for password hashing                             |
| `tls_certificate`              | (empty)                 | Path to TLS certificate (enables HTTPS)                                      |
| `tls_key`                      | (empty)                 | Path to TLS private key                                                      |
| `tls_root`                     | (empty)                 | Path to TLS root CA (for cluster mutual TLS)                                 |
| `cluster_token`                | `cluster`               | Shared secret for cluster inter-node auth                                    |
| `cluster`                      | `[]`                    | List of cluster node URLs, e.g. `["http://node1:3000", "http://node2:3000"]` |
| `cluster_heartbeat_timeout_ms` | `1000`                  | Raft heartbeat interval                                                      |
| `cluster_term_timeout_ms`      | `3000`                  | Raft term/election timeout                                                   |
| `cluster_election_factor_ms`   | `1000`                  | Per-node election delay multiplier (node_id × factor)                        |
| `cluster_max_log_entries`      | `1000`                  | Max Raft log entries before pruning                                          |
| `cluster_max_chunk_size`       | `65536`                 | Max bytes per resync chunk                                                   |
| `token_expiry_seconds`         | `3600`                  | Auth token TTL (range: 60–86400)                                             |
| `sync_mode`                    | `none`                  | Disk sync strategy: `none` (OS-managed) or `commit` (fsync on every commit)  |

## Cluster / Raft

The server supports a Raft-based cluster for high availability. Key characteristics:

- **Leader election**: Uses PreVote → Vote protocol. Election timeout is staggered per node (`cluster_election_factor_ms × node_id`) to reduce split votes.
- **Log replication**: The leader replicates `ClusterAction` log entries to followers. Actions include all mutating API operations (insert, remove, user/db management).
- **Commit rule**: A log entry is committed once a majority of nodes (⌊N/2⌋ + 1) acknowledge it.
- **Resync**: Nodes that fall behind can fully resync databases from the leader. The leader sends databases in chunks (`cluster_max_chunk_size`).
- **Single-node cluster**: Setting `cluster: ["http://localhost:3000"]` with a single URL runs a "cluster of one" — Raft runs but auto-commits immediately.
- **Cluster routes**: `/api/v1/cluster/*` endpoints propagate login/logout across all nodes. Regular `/api/v1/db/*` writes are automatically replicated by the Raft layer.
- **State machine**: Leader → Follower → Candidate → Election cycle. Followers redirect writes to the leader (or the client SDK follows the redirect).

### Cluster setup example

Three-node cluster config for node 1:

```yaml
bind: :::3001
address: http://node1:3001
admin: admin
cluster_token: my_secret_token
cluster: ["http://node1:3001", "http://node2:3002", "http://node3:3003"]
```

Each node lists the same `cluster` array. The node identifies itself by finding its own `address` in the list.

## API endpoints

The server exposes a REST API at `/api/v1` (or `{basepath}/api/v1`). Four endpoint families:

### Admin (`/api/v1/admin/...`) — requires admin token

**Database**: `/admin/db/{owner}/{db}/...`

- `POST /add` — create database (body: `DbKind`)
- `GET /audit` — database audit log
- `POST /backup` — create backup
- `POST /clear` — clear resources (audit, backup) (body: `ClearAudit | ClearBackup | ClearAll`)
- `POST /convert` — change storage type
- `POST /copy` — copy database (body: `{new_name}`)
- `DELETE /delete` — permanently delete
- `POST /exec`, `POST /exec_mut` — execute queries
- `POST /optimize` — defragment storage
- `POST /optimize_shrink_to_fit` — defragment and shrink file
- `DELETE /remove` — disassociate from server (keep data files)
- `POST /rename` — rename/move database
- `POST /restore` — restore from backup
- `POST /rollback` — rollback to previous WAL state
- `GET /user/list` — list database users
- `PUT /user/{username}/add` — add database user (body: `DbUserRole`)
- `DELETE /user/{username}/remove` — remove database user

**User**: `/admin/user/...`

- `POST /{username}/add` — create user (body: `{password}`)
- `PUT /{username}/change_password` — change password
- `DELETE /{username}/delete` — delete user and owned databases
- `GET /list` — list all users and sessions
- `POST /{username}/logout` — logout user
- `POST /{username}/logout?session={session}` — logout specific session
- `POST /logout_all` — logout all users

**Server**: `/admin/...`

- `POST /shutdown` — shutdown server
- `POST /set_log_level?new_level={level}` — set log level
- `GET /status` — server status and metrics (`AdminStatus`)

### User (`/api/v1/user/...`) — requires user token

- `POST /login` — authenticate, returns token string
- `POST /logout` — logout current session
- `POST /logout?session=others|all|{id}` — logout other/all/specific sessions
- `PUT /change_password` — change own password
- `GET /status` — own status and sessions (`UserStatus`)

### Database (`/api/v1/db/{owner}/{db}/...`) — requires user token + role

Same structure as admin database routes but scoped by ownership and roles:

- **Read** role: `exec`, `audit`
- **Write** role: `exec_mut` + everything Read can do
- **Admin** role: all operations including `backup`, `restore`, `convert`, `optimize`, user management

### Cluster (`/api/v1/cluster/...`) — propagates across nodes

- `POST /user/login` — cluster-wide login
- `POST /user/logout` — cluster-wide logout (supports `?session=` variants)
- `POST /admin/user/{username}/logout` — admin logout across cluster
- `POST /admin/user/logout_all` — logout all users across cluster
- `GET /status` — cluster node statuses (`Vec<ClusterStatus>`)

### Health

- `GET /api/v1/status` — health check, no auth required

## Rust client SDK (`agdb_api::AgdbApi`)

The client wraps all API endpoints as typed async methods. Key patterns:

```rs
use agdb_api::{AgdbApi, ReqwestClient, DbKind};
use agdb::QueryBuilder;

// Connect
let mut client = AgdbApi::new(ReqwestClient::new(), "localhost:3000");

// Authenticate
client.user_login("admin", "admin").await?;

// Create database
client.db_add("owner", "my_db", DbKind::Mapped).await?;

// Execute queries (immutable)
let queries = vec![
    QueryBuilder::search().from("root").query().into(),
];
let (status, results) = client.db_exec("owner", "my_db", &queries).await?;

// Execute queries (mutable)
let queries = vec![
    QueryBuilder::insert().nodes().aliases(["root"]).query().into(),
    QueryBuilder::insert().nodes().count(5).query().into(),
];
let (status, results) = client.db_exec_mut("owner", "my_db", &queries).await?;

// List databases
let (status, dbs) = client.db_list().await?;
```

### Method naming convention

Methods follow the pattern `{family}_{resource}_{action}`:

- `admin_db_add`, `admin_db_exec_mut`, `admin_user_add`
- `db_add`, `db_exec`, `db_exec_mut`, `db_backup`, `db_restore`
- `cluster_user_login`, `cluster_status`
- `user_login`, `user_logout`, `user_status`

### Authentication flow

1. Call `user_login("user", "pass")` — token is stored automatically in `client.token`.
2. Subsequent requests include the token as `Authorization: Bearer <token>`.
3. Token expires after `token_expiry_seconds` (server config). Re-login on 401.
4. For cluster-wide sessions, use `cluster_user_login` instead.

## Best practices for agentic query construction

When building queries programmatically (e.g. from an AI agent or automation):

### DO: Use the Rust client SDK

Always use `agdb_api::AgdbApi` with `QueryBuilder`. Never construct raw JSON for queries — the query format has nested enums and conditions that are extremely error-prone to build manually.

### DO: Use typed queries

```rs
#[derive(Debug, agdb::DbType)]
struct User {
    username: String,
    email: String,
}

// Insert typed data
let users = vec![User { username: "alice".into(), email: "alice@ex.com".into() }];
let result = client.db_exec_mut("owner", "db", &[
    QueryBuilder::insert().nodes().values(&users).query().into(),
]).await?;

// Query typed data back
let result = client.db_exec("owner", "db", &[
    QueryBuilder::select()
        .elements::<User>()
        .search()
        .from("users")
        .where_().key("username").value("alice")
        .query()
        .into(),
]).await?;
let users: Vec<User> = result.1[0].clone().try_into()?;
```

### DO: Batch queries in a single exec call

Multiple queries in one `exec_mut` call run atomically:

```rs
let queries = vec![
    QueryBuilder::insert().nodes().aliases(["users"]).query().into(),
    QueryBuilder::insert().nodes().values(&users).query().into(),
    // Use result of query [0] to link edges
];
let results = client.db_exec_mut("owner", "db", &queries).await?.1;
// results[0] = alias node, results[1] = user nodes
```

### DO: Use indexes for frequently queried keys

```rs
// Create index (once)
client.db_exec_mut("owner", "db", &[
    QueryBuilder::insert().index("username").query().into(),
]).await?;

// Fast lookup via index
client.db_exec("owner", "db", &[
    QueryBuilder::select()
        .elements::<User>()
        .search()
        .index("username")
        .value("alice")
        .query()
        .into(),
]).await?;
```

### DON'T: Use curl with raw JSON

The query serialization format is complex and undocumented for direct JSON construction. Queries contain deeply nested enums (`QueryType`, `QueryCondition`, `Comparison`, etc.) that are practically impossible to construct correctly by hand. Always use the typed builders.

### DON'T: Forget `.query()` at the end of builder chains

Every builder chain must be terminated with `.query()` and then `.into()` for batch execution.

### DON'T: Mix up `exec` and `exec_mut`

- `exec` is for `select` and `search` queries only.
- `exec_mut` is for `insert` and `remove` queries.
- Using the wrong one will result in a compile error (Rust) or a runtime 400 error (API).
