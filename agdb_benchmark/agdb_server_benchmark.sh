rm -f agdb_benchmark.yaml
rm -rf agdb_server_data

cargo build -r -p agdb_server
cargo run -r -p agdb_server > agdb_server_benchmark.log 2>&1 & 
cargo run -r -p agdb_benchmark -- agdb_server_benchmark.yaml

token=$(curl -X POST http://localhost:3000/api/v1/user/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}')
curl -H "Authorization: Bearer $token" -X POST http://localhost:3000/api/v1/admin/shutdown

rm -rf agdb_server_data

exit $error_code
