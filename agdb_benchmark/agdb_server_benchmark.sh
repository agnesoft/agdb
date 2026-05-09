set -euo pipefail

rm -f agdb_benchmark.yaml
rm -rf agdb_server_data

cargo build -r -p agdb_server
cargo run -r -p agdb_server > agdb_server_benchmark.log 2>&1 &
server_pid=$!

cleanup() {
  kill "$server_pid" 2>/dev/null || true
  wait "$server_pid" 2>/dev/null || true
  rm -rf agdb_server_data
}

trap cleanup EXIT

for _ in $(seq 1 20); do
  if curl -fsS http://localhost:3000/api/v1/status >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

curl -fsS http://localhost:3000/api/v1/status >/dev/null

cargo run -r -p agdb_benchmark -- agdb_server_benchmark.yaml

token=$(curl -fsS -X POST http://localhost:3000/api/v1/user/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}')

curl -fsS -H "Authorization: Bearer $token" \
  -X POST http://localhost:3000/api/v1/admin/shutdown
