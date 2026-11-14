#!/usr/bin/env bash
# Builds the agdb_server container image (with the studio) and smoke tests a 3 node cluster of it.
set -euo pipefail
cd "$(dirname "$0")/.."

docker build --pull -t agnesoft/agdb:dev -f agdb_server/containerfile .
trap 'docker compose -f agdb_server/compose.yaml down --volumes' EXIT
docker compose -f agdb_server/compose.yaml up --wait
sleep 5

curl -fsk https://localhost:3000/studio > /dev/null
curl -fsk https://localhost:3000/api/v1/cluster/status | grep -q '"leader":true'

token=$(curl -fsk -X POST https://localhost:3002/api/v1/cluster/user/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}' | tr -d '"')

for port in 3002 3000 3001; do
    curl -sk -H "Authorization: Bearer $token" -X POST "https://localhost:$port/api/v1/admin/shutdown"
done
