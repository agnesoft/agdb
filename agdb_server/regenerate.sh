#!/usr/bin/env bash
# The OpenAPI spec and the test queries the API clients are generated from.
set -euo pipefail
cargo test -p agdb_server api::tests::openapi -- --exact
cargo test -p agdb_server api::tests::test_queries -- --exact
