# AGENTS.md

The Agnesoft Graph Database (aka agdb) is a graph database. The main components of this repository are `agdb` (rust package) that is the database itself. The `agdb_server` (rust package) is the server version of the database. API client packages are available for Rust, TypeScript, and PHP. The OpenAPI specification is located at `agdb_server/openapi.json`.

# Available commands

- OpenAPI,API refresh and version bump: `cargo run -r -p agdb_ci`. Run when agdb_server/src/api.rs, agdb_server/openapi.json or Version file change.

## Rust

- Build package: `cargo build -r --all-features -p <package>`
- Build all: `cargo build -r --all-features`
- Format `cargo fmt`
- Test package: `cargo test -r --all-features -p <package>`
- Test all: `cargp test -r --all-features`
- Test debug: `cargo test --all-features -p <package>`
- Lint package: `cargo clippy --all-features -p <package>`
- Lint all: `cargo clippy --all-features`

## TypeScript

- Build: `pnpm run build --filter <package>`
- Format: `pnpm run format --filter <package>`
- Test: `pnpm run test --filter <package>`
- Run e2e tests: `pnpm run test:e2e --filter <package>`
- Lint: `pnpm run lint --filter <package>`

## PHP

- Test: `cd agdb_api/php/ && ./ci.sh coverage`
- Lint: `cd agdb_api/php/ && ./ci.sh analyse`
- Format: `cd agdb_api/php/ && ./ci.sh format`

# Available packages

The packages are curated lists of main packages. They contain exact names of packages directly usable in the available commands.

## Rust

- agdb
- agdb_benchmarks
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
