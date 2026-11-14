#!/usr/bin/env bash
# Pushes the agdb_server image (with the studio) to Docker Hub and GHCR tagged $AGCI_VERSION,
# releases also as latest. The image is built once per commit, further calls (e.g. dev and the
# release in the same run) only retag it. Expects docker to be logged in to both registries.
set -euo pipefail

cd "$(dirname "$0")/../.."
BUILD="agnesoft/agdb:build-$(git rev-parse HEAD)"
TAGS=("$AGCI_VERSION")

if [[ "$AGCI_VERSION" != "dev" ]]; then
    TAGS+=(latest)
fi

if ! docker image inspect "$BUILD" >/dev/null 2>&1; then
    docker build --pull -t "$BUILD" -f agdb_server/containerfile .
fi

for IMAGE in agnesoft/agdb ghcr.io/agnesoft/agdb; do
    for TAG in "${TAGS[@]}"; do
        docker tag "$BUILD" "$IMAGE:$TAG"
        docker push "$IMAGE:$TAG"
    done
done
