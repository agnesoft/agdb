---
name: agdb-api
description: Navigate agdb API structure, endpoints, authentication, and client libraries. Use this skill when building API clients, writing server tests, understanding endpoint behavior, or working with API transpilations to other languages.
argument-hint: "[navigate|explain|test|generate] [endpoint family or use case]"
---

# agdb API Skill

Use this skill when:

- Understanding agdb server API structure and endpoints
- Writing tests for agdb_server routes
- Explaining API behavior to implement in client libraries
- Planning API client generators or transpilations
- Debugging authentication/authorization flows
- Building wrappers or SDKs for other languages
- Interacting with the server programmatically (always use the Rust SDK, never raw JSON)

Primary sources in this repository:

- `agdb_api/src/client.rs` — Rust API client (all endpoint methods)
- `agdb_server/openapi.json` — OpenAPI 3.1.0 specification (complete contract)
- `agdb_server/src/routes/` — Route implementations (auth, validation, errors)
- `agdb_server/tests/` — Integration tests
- `agdb_api/src/` — Rust client source package
- `agdb_api/typescript/` — TypeScript client package
- `agdb_api/php/` — PHP client package

## API organization

The agdb server exposes a REST API at `/api/v1` (or `{basepath}/api/v1` when `basepath` is configured) organized into four endpoint families:

### Admin routes (`/api/v1/admin/...`)

Require admin authentication. Manage all databases, users, and server state.

- **Database management**: `/admin/db/{owner}/{db}/...`
  - `POST /add` — Create database (body: `DbKind` — `Memory`, `Mapped`, `File`)
  - `GET /audit` — View database audit log → `DbAudit`
  - `POST /backup` — Create backup snapshot
  - `POST /clear` — Clear resources (body: `ClearAudit | ClearBackup | ClearAll`)
  - `POST /convert` — Change storage type (body: `DbKind`)
  - `POST /copy` — Copy database (body: `{new_name}`)
  - `DELETE /delete` — Permanently delete database and files
  - `POST /optimize` — Defragment storage, returns `ServerDatabase` with new size
  - `POST /optimize_shrink_to_fit` — Defragment and shrink file on disk
  - `DELETE /remove` — Disassociate from server (keep data files)
  - `POST /rename` — Rename/move database (body: `{new_name}`)
  - `POST /restore` — Restore from backup
  - `POST /rollback` — Rollback to previous WAL checkpoint
  - `POST /exec` — Execute immutable queries → `Vec<QueryResult>`
  - `POST /exec_mut` — Execute mutable queries → `Vec<QueryResult>`
  - `GET /user/list` — List database users → `Vec<DbUser>`
  - `PUT /user/{username}/add` — Add database user (body: `DbUserRole`)
  - `DELETE /user/{username}/remove` — Remove database user

- **User management**: `/admin/user/...`
  - `POST /{username}/add` — Create user (body: `{password}`)
  - `PUT /{username}/change_password` — Change password (body: `{password}`)
  - `GET /list` — List all users and sessions → `Vec<UserStatus>`
  - `POST /{username}/logout` — Logout user (all sessions on this node)
  - `POST /{username}/logout?session={session}` — Logout specific session
  - `POST /logout_all` — Logout all users on this node
  - `DELETE /{username}/delete` — Delete user and all owned databases

- **Server management**: `/admin/...`
  - `POST /shutdown` — Graceful server shutdown
  - `POST /set_log_level?new_level={level}` — Set log level (`trace`, `debug`, `info`, `warn`, `error`, `off`)
  - `GET /status` — Server status and metrics → `AdminStatus`

### User routes (`/api/v1/user/...`)

Require user authentication. User manages own account.

- `POST /login` — Authenticate → token string (body: `{username, password}`)
- `POST /logout` — Logout current session
- `POST /logout?session=others` — Logout all other sessions
- `POST /logout?session=all` — Logout all sessions
- `POST /logout?session={session}` — Logout specific session by id
- `PUT /change_password` — Change own password (body: `{password}`)
- `GET /status` — Own status and sessions → `UserStatus`

### Database routes (`/api/v1/db/{owner}/{db}/...`)

Require user authentication and appropriate role.

Same structure as admin database routes but scoped to owner/accessible databases:

| Role      | Allowed operations                                                                                            |
| --------- | ------------------------------------------------------------------------------------------------------------- |
| **Read**  | `exec`, `audit`                                                                                               |
| **Write** | Everything Read + `exec_mut`                                                                                  |
| **Admin** | Everything Write + `backup`, `restore`, `rollback`, `convert`, `optimize`, `clear`, `user/add`, `user/remove` |

Additionally:

- `POST /add` — Create database (owner must be self)
- `DELETE /delete` — Delete (owner only)
- `DELETE /remove` — Remove (owner only)
- `POST /rename` — Rename (owner only)
- `POST /copy` — Copy (owner scope)
- `GET /list` — List accessible databases → `Vec<ServerDatabase>` (no `{owner}/{db}` path params)

### Cluster routes (`/api/v1/cluster/...`)

Propagate operations across all cluster nodes. These endpoints replicate the action to every node via the Raft log.

- `POST /user/login` — Authenticate cluster-wide → token valid on all nodes
- `POST /user/logout` — Logout cluster-wide
- `POST /user/logout?session=others|all|{id}` — Logout variants across cluster
- `POST /admin/user/{username}/logout` — Admin logout user across cluster
- `POST /admin/user/{username}/logout?session={session}` — Admin logout specific session across cluster
- `POST /admin/user/logout_all` — Logout all users across cluster
- `GET /status` — Cluster node statuses → `Vec<ClusterStatus>`

### Health check

- `GET /api/v1/status` — Server health (no auth required), returns HTTP 200

## Authentication & authorization

### Token-based authentication

1. Call `POST /user/login` or `POST /cluster/user/login` with `{username, password}`.
2. Server returns a token string.
3. Include token in `Authorization: Bearer <token>` header for subsequent requests.
4. Token expires after `token_expiry_seconds` (configurable, default 3600, range 60–86400).
5. Expired tokens return `401 Unauthorized`.
6. Each login creates a new session; multiple sessions per user are allowed.

### Roles and permissions

Users can have roles on databases:

- **Admin**: Full control (all operations, including backup, restore, rollback, user management)
- **Write**: Read + modify data (exec_mut)
- **Read**: Query only (exec, audit)

Admin users have server-wide admin access and are required for user and server operations.

### Common error codes

| Code  | Meaning                                                     |
| ----- | ----------------------------------------------------------- |
| `200` | Success with a response body (GET, `exec`, `exec_mut`)      |
| `201` | Created (create-like POST/PUT operations)                   |
| `204` | No content (DELETE and other successful no-body operations) |
| `400` | Bad request (malformed input)                               |
| `401` | Unauthorized (missing/expired token, invalid credentials)   |
| `403` | Forbidden (insufficient permissions / wrong role)           |
| `404` | Not found (user, database, or resource does not exist)      |
| `461` | Password too short (<8 chars)                               |
| `462` | User name too short (<3 chars)                              |
| `463` | User already exists                                         |
| `464` | User not found                                              |
| `465` | Database already exists                                     |
| `467` | Invalid database name                                       |

## Client libraries

### Rust client (`agdb_api::AgdbApi`)

Hand-written, fully typed. Methods map directly to endpoints. **This is the recommended way to interact with the server programmatically.**

```rs
use agdb_api::{AgdbApi, ReqwestClient, DbKind};
use agdb::QueryBuilder;

let mut client = AgdbApi::new(ReqwestClient::new(), "localhost:3000");

// Authenticate
client.user_login("admin", "password").await?;

// Create database
client.db_add("owner", "my_db", DbKind::Mapped).await?;

// Execute queries
let queries = vec![
    QueryBuilder::insert().nodes().aliases(["root"]).query().into(),
    QueryBuilder::insert().nodes().count(5).query().into(),
];
let (status, results) = client.db_exec_mut("owner", "my_db", &queries).await?;
// results[0].result == 1, results[1].result == 5

// Read back
let (status, results) = client.db_exec("owner", "my_db", &[
    QueryBuilder::select().search().from("root").where_().node().query().into(),
]).await?;
```

**Location**: `agdb_api/src/client.rs`

**Package name**: `agdb_api`

**Cargo features**:

- `api` (default) — enables derive macros, tokio, and the full client
- `derive` — agdb derive macros re-exported

**Method naming convention**: `{family}_{resource}_{action}`

- `admin_db_add`, `admin_db_exec_mut`, `admin_user_add`
- `db_add`, `db_exec`, `db_exec_mut`, `db_backup`, `db_restore`
- `cluster_user_login`, `cluster_status`
- `user_login`, `user_logout`, `user_status`

**Custom HTTP client**: Implement the `HttpClient` trait to replace `ReqwestClient` with any HTTP library. The trait defines `delete`, `get`, `post`, `put` methods.

### TypeScript client

Generated from OpenAPI. Includes models and request types.

```ts
import { client } from "@agnesoft/agdb_api";

const api = client("http://localhost:3000");
await api.user_login({ username: "admin", password: "password" });
const dbs = await api.db_list();
```

**Location**: `agdb_api/typescript/src/`

**Package name**: `@agnesoft/agdb_api`

### PHP client

Generated from OpenAPI using openapi-generator.

```php
$api = new \Agnesoft\AgdbApi\Api\AgdbApi(
    new \GuzzleHttp\Client(),
    $config
);
$api->userLogin(['userLogin' => new UserLogin(['username' => 'admin', 'password' => 'password'])]);
```

**Location**: `agdb_api/php/lib/`

**Package name**: `agnesoft/agdb-api`

**URL**: https://packagist.org/packages/agnesoft/agdb_api

## Server configuration reference

The server reads a YAML-like config file (default: `agdb_server.yaml`). First run creates a default config.

| Key                            | Default                 | Description                                                    |
| ------------------------------ | ----------------------- | -------------------------------------------------------------- |
| `bind`                         | `:::3000`               | Socket bind address                                            |
| `address`                      | `http://localhost:3000` | Public address for cluster discovery                           |
| `basepath`                     | (empty)                 | URL path prefix (e.g. `/api`); auto-prepended `/` if missing   |
| `static_roots`                 | `[]`                    | Directories to serve as static files                           |
| `admin`                        | `admin`                 | Admin username (created on first run with password = username) |
| `log_level`                    | `info`                  | `trace`, `debug`, `info`, `warn`, `error`, `off`               |
| `log_body_limit`               | `10240`                 | Max bytes of request/response body to log                      |
| `request_body_limit`           | `10485760`              | Max request body size (10 MiB)                                 |
| `data_dir`                     | `agdb_server_data`      | Directory for databases and server state                       |
| `pepper_path`                  | (empty)                 | Path to 16-byte pepper file for password hashing               |
| `tls_certificate`              | (empty)                 | Path to TLS certificate (enables HTTPS)                        |
| `tls_key`                      | (empty)                 | TLS private key                                                |
| `tls_root`                     | (empty)                 | TLS root CA (for cluster mutual TLS)                           |
| `cluster_token`                | `cluster`               | Shared secret for cluster inter-node auth                      |
| `cluster`                      | `[]`                    | List of cluster node URLs                                      |
| `cluster_heartbeat_timeout_ms` | `1000`                  | Raft heartbeat interval                                        |
| `cluster_term_timeout_ms`      | `3000`                  | Raft term/election timeout                                     |
| `cluster_election_factor_ms`   | `1000`                  | Per-node election delay (node_id × factor)                     |
| `cluster_max_log_entries`      | `1000`                  | Max Raft log entries before pruning                            |
| `cluster_max_chunk_size`       | `65536`                 | Max bytes per resync chunk                                     |
| `token_expiry_seconds`         | `3600`                  | Auth token TTL (range: 60–86400)                               |
| `sync_mode`                    | `none`                  | `none` (OS-managed flush) or `commit` (fsync every commit)     |

## Cluster / Raft

The server supports Raft-based clustering for high availability:

- **Leader election**: PreVote → Vote protocol. Election timeout staggered per node (`node_id × cluster_election_factor_ms`) to reduce split votes.
- **Log replication**: Leader replicates `ClusterAction` entries (all mutating API operations) to followers.
- **Commit rule**: Entry committed once majority (⌊N/2⌋ + 1) acknowledge.
- **Write forwarding**: All mutable API requests on non-leader nodes are forwarded to the leader. Reads are served locally.
- **Resync**: Nodes that fall behind fully resync databases from the leader in chunks.
- **Single-node mode**: `cluster: ["http://localhost:3000"]` runs Raft but auto-commits immediately.
- **Cluster auth endpoints**: `/cluster/user/login` and `/cluster/user/logout` propagate sessions across all nodes; regular `/user/login` is node-local.

### Cluster setup

Each node needs its own config file. All nodes list the same `cluster` array. A node identifies itself by finding its `address` (+ `basepath`) in the list.

Node 1:

```yaml
bind: :::3001
address: http://node1:3001
admin: admin
cluster_token: my_secret
cluster: ["http://node1:3001", "http://node2:3002", "http://node3:3003"]
```

Node 2:

```yaml
bind: :::3002
address: http://node2:3002
admin: admin
cluster_token: my_secret
cluster: ["http://node1:3001", "http://node2:3002", "http://node3:3003"]
```

## Agentic best practices (programmatic server interaction)

### ALWAYS use the Rust SDK, never raw JSON

The query serialization format uses deeply nested Rust enums (`QueryType`, `QueryCondition`, `Comparison`, `CountComparison`, etc.) that are practically impossible to construct correctly as raw JSON. **Always use `agdb_api::AgdbApi` with `QueryBuilder`.**

### Sample: full agentic Rust program

```rs
use agdb::{DbType, QueryBuilder};
use agdb_api::{AgdbApi, ReqwestClient, DbKind};

#[derive(Debug, DbType)]
struct Evidence {
    identity: String,
    control_id: String,
    status: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Connect and authenticate
    let mut client = AgdbApi::new(ReqwestClient::new(), "localhost:3000");
    client.user_login("admin", "admin").await?;

    // 2. Create (or verify) database exists
    match client.db_add("admin", "evidence_db", DbKind::Mapped).await {
        Ok(_) => println!("Database created"),
        Err(e) if e.status == 465 => println!("Database already exists"),
        Err(e) => return Err(e.into()),
    }

    // 3. Set up schema: root node + index
    client.db_exec_mut("admin", "evidence_db", &[
        QueryBuilder::insert().nodes().aliases(["evidence_root"]).query().into(),
        QueryBuilder::insert().index("identity").query().into(),
        QueryBuilder::insert().index("control_id").query().into(),
    ]).await?;

    // 4. Insert data
    let records = vec![
        Evidence { identity: "app-1".into(), control_id: "SOX-001".into(), status: "pass".into() },
        Evidence { identity: "app-1".into(), control_id: "SOX-002".into(), status: "fail".into() },
        Evidence { identity: "app-2".into(), control_id: "SOX-001".into(), status: "pass".into() },
    ];

    let results = client.db_exec_mut("admin", "evidence_db", &[
        QueryBuilder::insert().nodes().values(&records).query().into(),
    ]).await?.1;

    // Link to root
    client.db_exec_mut("admin", "evidence_db", &[
        QueryBuilder::insert().edges().from("evidence_root").to(results[0].ids()).query().into(),
    ]).await?;

    // 5. Query: find all evidence for identity "app-1"
    let (_, results) = client.db_exec("admin", "evidence_db", &[
        QueryBuilder::select()
            .elements::<Evidence>()
            .search()
            .index("identity")
            .value("app-1")
            .query()
            .into(),
    ]).await?;

    let evidence: Vec<Evidence> = results[0].clone().try_into()?;
    for e in &evidence {
        println!("{}: {} = {}", e.identity, e.control_id, e.status);
    }

    // 6. Update: set status to "remediated" for SOX-002
    let (_, to_update) = client.db_exec("admin", "evidence_db", &[
        QueryBuilder::search()
            .from("evidence_root")
            .where_()
            .key("control_id").value("SOX-002")
            .and()
            .key("identity").value("app-1")
            .query()
            .into(),
    ]).await?;

    client.db_exec_mut("admin", "evidence_db", &[
        QueryBuilder::insert()
            .values_uniform([("status", "remediated").into()])
            .ids(to_update[0].ids())
            .query()
            .into(),
    ]).await?;

    // 7. Backup
    client.db_backup("admin", "evidence_db").await?;

    Ok(())
}
```

### Error handling

```rs
match client.db_exec_mut("owner", "db", &queries).await {
    Ok((status, results)) => {
        // status is the HTTP status code (200)
        // results is Vec<QueryResult>
    }
    Err(e) => {
        // e.status: HTTP status code (401, 403, 404, etc.)
        // e.description: error message from server
        eprintln!("API error {}: {}", e.status, e.description);
    }
}
```

### Batch operations for atomicity

All queries in a single `exec_mut` call execute atomically. Use this to ensure consistency:

```rs
// These three operations are atomic
let results = client.db_exec_mut("owner", "db", &[
    QueryBuilder::insert().nodes().aliases(["container"]).query().into(),
    QueryBuilder::insert().nodes().values(&items).query().into(),
    // If any query fails, all are rolled back
]).await?.1;

// Chain results: use ids from first query in second
client.db_exec_mut("owner", "db", &[
    QueryBuilder::insert().edges().from("container").to(results[1].ids()).query().into(),
]).await?;
```

## Testing patterns

### Unit tests in agdb_server

Located in `agdb_server/tests/` and route modules.

- **Route tests**: Test individual endpoint handlers with mocked dependencies.
- **Auth tests**: Verify token validation, expiration, role checks.
- **Error tests**: Confirm correct error codes for edge cases.

Example pattern:

```rs
#[tokio::test]
async fn test_endpoint_success() {
    let api = setup_test_api().await;
    let result = api.some_endpoint(...).await;
    assert_eq!(result.status(), 200);
}

#[tokio::test]
async fn test_endpoint_unauthorized() {
    let api = setup_test_api_without_token().await;
    let result = api.some_endpoint(...).await;
    assert_eq!(result.status(), 401);
}
```

### Integration tests

Test end-to-end workflows:

- User lifecycle (create, login, change password, delete)
- Database workflows (create, execute queries, backup, restore)
- Permission enforcement (role-based access)
- Token management (expiry, logout)

## OpenAPI specification

The `agdb_server/openapi.json` is the single source of truth for the API contract.

### Key sections

- **info**: Title, version, description
- **servers**: Base URL (`http://localhost:3000`)
- **paths**: All endpoints with methods, parameters, responses
- **components/schemas**: All request/response types and enums
- **components/securitySchemes**: `Token` bearer authentication

### Regenerating OpenAPI

The spec is regenerated when API routes, response types, or error codes change.

**Command**: `cargo run -r -p agdb_ci`

## API transpilation workflow

When adding support for new languages:

1. **Ensure openapi.json is current** — Run `cargo run -r -p agdb_ci`
2. **Generate client** — Use openapi-generator:
   ```bash
   openapi-generator-cli generate \
     -i agdb_server/openapi.json \
     -g <language> \
     -o agdb_api/<language>/ \
     --additional-properties packageName=agdb_api
   ```
3. **Customize if needed** — Hand-write convenience methods, add documentation, handle edge cases
4. **Add tests** — Verify client works against running server
5. **Package and publish** — Integrate into CI/CD

## Key design principles

1. **Single OpenAPI spec drives all clients** — Consistency across languages
2. **Token-based auth with expiry** — Stateless servers enable horizontal scaling
3. **Role-based access control** — Fine-grained permissions per database (Read/Write/Admin)
4. **Immutable and mutable query separation** — `exec` vs `exec_mut` provides compile-time safety
5. **Descriptive error codes** — Custom 46x codes for domain-specific errors
6. **Cluster-aware routes** — `/cluster/...` propagates operations; `/user/...` and `/db/...` are node-local (writes auto-replicated by Raft)
7. **Atomic batch execution** — Multiple queries in one `exec_mut` call run in a single transaction

## File organization reference

```
agdb_api/
  src/
    client.rs              ← Hand-written Rust client (all methods)
    api_types.rs           ← Shared types (UserStatus, DbUser, ServerDatabase, etc.)
    api_types/
      config_impl.rs       ← Server config struct and serialization
    api_error.rs           ← AgdbApiError struct
    api_result.rs          ← AgdbApiResult type alias
    http_client.rs         ← HttpClient trait + ReqwestClient implementation
    lib.rs                 ← Crate root, re-exports
  typescript/
    src/                   ← Generated TypeScript client
  php/
    lib/                   ← Generated PHP client
    lib/Model/             ← Schema models

agdb_server/
  src/
    api.rs                 ← utoipa OpenAPI schema registration
    config.rs              ← Config parsing
    cluster.rs             ← Cluster coordination (Raft integration)
    cluster_log.rs         ← Raft log persistence
    raft.rs                ← Generic Raft implementation
    routes/
      admin/
        db.rs              ← Admin database routes
        user.rs            ← Admin user routes
      cluster.rs           ← Cluster routes
      db/                  ← User database routes
      user.rs              ← User auth routes
  openapi.json             ← OpenAPI specification
  tests/                   ← Integration tests
```

## Validation checklist for new endpoints

Before adding new endpoints:

- [ ] Method (GET, POST, PUT, DELETE) aligns with semantics
- [ ] Path parameters in `{braces}` are documented
- [ ] Query parameters with `?key=value` are clearly named
- [ ] Request body (if any) is a well-defined schema
- [ ] All response codes (200, 201, 204, 4xx, 5xx) are listed
- [ ] Security requirement (`Token` auth) is specified
- [ ] Operation id is unique and matches handler name
- [ ] Description explains intent and side effects
- [ ] OpenAPI spec is regenerated with `cargo run -r -p agdb_ci`
- [ ] Rust client method added in `agdb_api/src/client.rs`
