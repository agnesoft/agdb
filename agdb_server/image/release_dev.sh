#!/usr/bin/env bash
# The dev image, release.sh tags it $AGCI_VERSION (dev) only.
set -euo pipefail
exec bash "$(dirname "$0")/release.sh"
