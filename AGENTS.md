# AGENTS.md

The Agnesoft Graph Database (aka agdb) is a graph database. The main components of this repository are `agdb` (rust package) that is the database itself. The `agdb_server` (rust package) is the server version of the database. API client packages are available for Rust, TypeScript, and PHP. The OpenAPI specification is located at `agdb_server/openapi.json`.

# Setup

- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Install pnpm: `npm i -g pnpm`. This assumes `npm` is installed & available.
- Install PHP and Composer. Only needed for PHP API client. Use the `php-composer-setup` skill in `.github/skills/php-composer-setup/SKILL.md`.
- Install playwright: `pnpm exec playwright install`. Only for TypeScript e2e tests.

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
- Run e2e tests: `pnpm run test:e2e --filter <package>`
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
