rm -f agdb_server.yaml
rm -rf agdb_server_data
cargo build -r -p agdb_server
cargo run -r -p agdb_server &

pnpm exec vitest run --coverage
error_code=$?

token=$(curl -X POST http://localhost:3000/api/v1/user/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}')
curl -H "Authorization: Bearer $token" -X POST http://localhost:3000/api/v1/admin/shutdown

rm -f agdb_server.yaml
rm -rf agdb_server_data

exit $error_code
