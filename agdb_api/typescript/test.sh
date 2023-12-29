cd ../../agdb_server
cargo run --release &
cd ../agdb_api/typescript
npx vitest run --coverage
error_code=$?
token=$(curl -X POST http://localhost:3000/api/v1/user/admin/login -H "Content-Type: application/json" -d '{"password":"admin"}')
curl -H "Authorization: Bearer $token" -X POST http://localhost:3000/api/v1/admin/shutdown
exit $error_code