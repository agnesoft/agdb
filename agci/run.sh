#!/usr/bin/env bash
# The versions, API clients and test queries are up to date (the check fails if this changes files).
set -euo pipefail
cargo run -r -p agci
# The Rust printed from the agsource IR of agdb and agdb_api is the code as written.
cargo run -r -p agci -- roundtrip agdb agdb_api
