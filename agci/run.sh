#!/usr/bin/env bash
# The versions, API clients and test queries are up to date (the check fails if this changes files).
set -euo pipefail
cargo run -r -p agci
