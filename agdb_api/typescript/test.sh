cd ../../agdb_server
rm -f agdb_server.yaml
rm -f .agdb_server.agdb
rm -f agdb_server.agdb
cargo build --release
cargo run --release &
cd ../agdb_api/typescript

npx vitest run --coverage

error_code=$?
token=$(curl -X POST http://localhost:3000/api/v1/user/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}')
curl -H "Authorization: Bearer $token" -X POST http://localhost:3000/api/v1/admin/shutdown
rm -f agdb_server.yaml
rm -f .agdb_server.agdb
rm -f agdb_server.agdb
exit $error_code
