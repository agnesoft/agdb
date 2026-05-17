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

Primary sources in this repository:

- `agdb_api/src/client.rs` — Rust API client (all endpoint methods)
- `agdb_server/openapi.json` — OpenAPI 3.1.0 specification (complete contract)
- `agdb_server/src/routes/` — Route implementations (auth, validation, errors)
- `agdb_server/tests/` — Integration tests
- `agdb_api/src/` — Rust client source package
- `agdb_api/typescript/` — TypeScript client package
- `agdb_api/php/` — PHP client package

## API organization

The agdb server exposes a REST API at `/api/v1` organized into four endpoint families:

### Admin routes (`/api/v1/admin/...`)

Require admin authentication. Manage all databases, users, and server state.

- **Database management**: `/admin/db/{owner}/{db}/...`
  - `POST /add` — Create database
  - `GET /audit` — View database audit log
  - `POST /backup` — Create backup
  - `POST /clear` — Clear resources (audit, backup)
  - `POST /convert` — Change storage type
  - `POST /copy` — Copy database
  - `DELETE /delete` — Permanently delete
  - `POST /optimize` — Defragment storage
  - `DELETE /remove` — Disassociate from server (keep data)
  - `POST /rename` — Rename/move database
  - `POST /restore` — Restore from backup
  - `POST /exec`, `POST /exec_mut` — Execute queries
  - `GET /user/list` — List database users
  - `PUT /user/{username}/add` — Add database user
  - `DELETE /user/{username}/remove` — Remove database user

- **User management**: `/admin/user/...`
  - `POST /{username}/add` — Create user
  - `PUT /{username}/change_password` — Change password
  - `GET /list` — List all users and sessions
  - `POST /{username}/logout` — Logout user
  - `POST /{username}/logout?session={session}` — Logout specific session
  - `POST /logout_all` — Logout all users
  - `DELETE /{username}/delete` — Delete user and owned databases

- **Server management**: `/admin/...`
  - `POST /shutdown` — Shutdown server
  - `POST /set_log_level?new_level={level}` — Set log level
  - `GET /status` — Server status and metrics

### User routes (`/api/v1/user/...`)

Require user authentication. User manages own account and databases they own.

- `POST /login` — Authenticate and get token
- `POST /logout` — Logout current session
- `POST /logout?session=others` — Logout other sessions
- `POST /logout?session=all` — Logout all sessions
- `POST /logout?session={session}` — Logout specific session
- `PUT /change_password` — Change own password
- `GET /status` — Get own status and sessions

### Database routes (`/api/v1/db/{owner}/{db}/...`)

Require user authentication and appropriate role (Read, Write, Admin).

Similar structure to admin database routes but scoped to owner/accessible databases:

- `POST /add` — Create database (owner must be self)
- `GET /audit` — View audit log (requires database access)
- `POST /backup` — Create backup (admin role required)
- `POST /clear` — Clear resources (admin role required)
- `POST /convert` — Convert storage type (admin role required)
- `POST /copy` — Copy database (owner scope)
- `DELETE /delete` — Delete database (owner only)
- `GET /exec`, `POST /exec_mut` — Execute queries
- `POST /optimize` — Optimize storage (admin role required)
- `DELETE /remove` — Remove database (owner only)
- `POST /rename` — Rename database (owner only)
- `POST /restore` — Restore from backup (admin role required)
- `GET /user/list` — List database users
- `PUT /user/{username}/add` — Add database user (admin role required)
- `DELETE /user/{username}/remove` — Remove database user (admin role required)

### Cluster routes (`/api/v1/cluster/...`)

Manage cluster state and login across cluster.

- `POST /user/login` — Authenticate cluster-wide
- `POST /user/logout` — Logout cluster-wide
- `POST /user/logout?session=others` — Logout other cluster sessions
- `POST /user/logout?session=all` — Logout all cluster sessions
- `POST /user/logout?session={session}` — Logout specific cluster session
- `POST /admin/user/{username}/logout` — Logout user across cluster
- `POST /admin/user/{username}/logout?session={session}` — Logout user session
- `POST /admin/user/logout_all` — Logout all users across cluster
- `GET /status` — Cluster node status

### Health check

- `GET /api/v1/status` — Server health (no auth required)

## Authentication & authorization

### Token-based authentication

1. Call `/user/login` or `/cluster/user/login` with username and password.
2. Server returns a token string.
3. Store token in `Authorization: Bearer <token>` header or equivalent.
4. Token expires after configured duration (default 3600 seconds).
5. Expired tokens return `401 Unauthorized`.

### Roles and permissions

Users can have roles on databases:

- **Admin**: Full control (all operations, including backup and restore)
- **Write**: Read + modify data (exec_mut)
- **Read**: Query only (exec, audit)

Admin users have server-wide admin access and are required for user and server operations.

### Common error codes

| Code | Meaning |
|------|---------|
| `200` | Success with a response body (for example GET, `exec`, `exec_mut`) |
| `201` | Created (create-like POST/PUT operations) |
| `204` | No content (DELETE and other successful no-body operations) |
| `400` | Bad request (malformed input) |
| `401` | Unauthorized (missing/expired token, invalid credentials) |
| `403` | Forbidden (insufficient permissions) |
| `404` | Not found (user, database, or resource does not exist) |
| `461` | Password too short (<8 chars) |
| `462` | User name too short (<3 chars) |
| `463` | User already exists |
| `464` | User not found |
| `465` | Database already exists |
| `467` | Invalid database name |

## Client libraries

The agdb_api provides generated and hand-written clients in multiple languages:

### Rust client (`agdb_api::AgdbApi`)

Hand-written, fully typed. Methods map directly to endpoints.

```rs
let mut api = AgdbApi::new(ReqwestClient::new(), "http://localhost:3000");
api.user_login("admin", "password").await?;
let (status, dbs) = api.db_list().await?;
api.db_exec("owner", "db", &[query]).await?;
```

**Location**: `agdb_api/rust/src/client.rs`

**Package name**: `agdb_api`

### TypeScript client

Generated from OpenAPI. Includes models and request types.

```ts
const api = client("http://localhost:3000");
await api.user_login({ username: "admin", password: "password" });
const dbs = await api.db_list();
```

**Location**: `agdb_api/typescript/src/`

**Package name**: `@agnesoft/agdb_api`

### PHP client

Generated from OpenAPI using openapi-generator.

```php
$api = new \Agnesoft\AgdbApi\Api\UserApi(
    new \GuzzleHttp\Client(),
    $config
);
$api->userLogin(['userLogin' => new UserLogin(['username' => 'admin', 'password' => 'password'])]);
```

**Location**: `agdb_api/php/lib/`

**Package name**: `agnesoft/agdb-api`

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

- **info**: Title, version (`0.12.10`), description
- **servers**: Base URL (`http://localhost:3000`)
- **paths**: All endpoints with methods, parameters, responses
- **components/schemas**: All request/response types and enums
- **components/securitySchemes**: `Token` bearer authentication

### Using OpenAPI

1. **Code generation**: openapi-generator can generate clients in 50+ languages.
2. **Documentation**: Generate interactive docs with Swagger UI or ReDoc.
3. **Validation**: Use tools to validate requests/responses against schema.
4. **Contract testing**: Verify implementations match specification.

### Regenerating OpenAPI

The OpenAPI spec is regenerated when:

- API routes change (`agdb_server/src/routes/`)
- Response types change
- Error codes change

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
3. **Role-based access control** — Fine-grained permissions per database
4. **Immutable and mutable query separation** — Compile-time safety and clear intent
5. **Descriptive error codes** — Custom 46x codes for domain-specific errors (password length, user exists, etc.)
6. **Cluster-aware routes** — Some operations span cluster (`/cluster/...`), others are node-local

## Common workflows

### Login and list databases

```
POST /api/v1/user/login          → token
GET /api/v1/db/list (+ token)     → [ServerDatabase]
```

### Execute queries

```
POST /api/v1/db/{owner}/{db}/exec     → [QueryResult]
POST /api/v1/db/{owner}/{db}/exec_mut → [QueryResult]
```

### Manage backups

```
POST /api/v1/db/{owner}/{db}/backup  → creates backup
POST /api/v1/db/{owner}/{db}/restore → swap with backup
```

### User management (admin)

```
POST /api/v1/admin/user/{name}/add                → create user
PUT /api/v1/admin/user/{name}/change_password     → set password
GET /api/v1/admin/user/list                        → all users + sessions
POST /api/v1/admin/user/{name}/logout              → logout user
DELETE /api/v1/admin/user/{name}/delete            → delete user
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

## File organization reference

```
agdb_api/
  rust/
    src/client.rs              ← Hand-written Rust client
    src/api_types.rs           ← Shared types (UserStatus, DbUser, etc.)
  typescript/
    src/                       ← Generated TypeScript client
    src/openapi.d.ts           ← Type definitions
  php/
    lib/                       ← Generated PHP client
    lib/Model/                 ← Schema models

agdb_server/
  src/
    routes/                    ← Route handlers by family
      admin/db.rs              ← Admin database routes
      admin/user.rs            ← Admin user routes
      cluster.rs               ← Cluster routes
      db/                      ← User database routes
      user.rs                  ← User auth routes
  openapi.json                 ← OpenAPI specification
  tests/                       ← Integration tests
```
