# AGENTS.md

The Agnesoft Graph Database (aka agdb) is a graph database. Main components of the repository are `agdb` rust package that is the database itself. The `agdb_server` is the server version of the database itself using `agdb`. API client packages are available for Rust, TypeScript and PHP and the OpenAPI spec which is located under `agdb_server/openapi.json`.

# Available commands

- Rust: `cargo <command> --all-features -r -p <package>`
- TypeScript: `pnpm run <command> --filter <package>`
- PHP: `cd agdb_api/php/ && ./ci.sh <command>`
- OpenAPI & API refresh: `cargo run -r -p agdb_ci`. Only run when agdb_server/src/api.rs or agdb_server/openapi.json change.

## Rust <command>

- build
- test
- clippy

## TypeScript <command>

- build
- format
- test
- test:e2e
- lint

## PHP <command>

- coverage
- analysis
- format

# Available <package>

All packages are exact names of packages directly usable in the available commands.

## Rust <package>

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

## TypeScript <package>

- agdb_studio
- @agnesoft/agdb_api
- agdb_web
- examples_server_client
