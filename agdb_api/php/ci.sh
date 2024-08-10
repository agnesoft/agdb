function coverage() {
    rm -f agdb_server.yaml
    rm -f .agdb_server.agdb
    rm -f agdb_server.agdb
    rm -rf agdb_server_data

    echo "bind: \"0.0.0.0:3000\"
address: localhost:3000
basepath: \"\"
admin: admin
data_dir: agdb_server_data
cluster: []" > agdb_server.yaml

    cargo build --release -p agdb_server
    cargo run --release -p agdb_server &
    sleep 3
    
    local output
    output=$(XDEBUG_MODE=coverage ./vendor/bin/phpunit tests --coverage-filter src/ --coverage-text --coverage-html coverage/)
    local error_code=$?
    echo "ERROR CODE: $error_code"
    echo "$output"
    echo ""

    if echo "$output" | grep "Lines:" | head -1 | grep -q "100.00%"; then
        echo "Line coverage OK";
    else
        echo "Insufficient line coverage";
        error_code=2
    fi
    
    if echo "$output" | grep "Methods:" | head -1 | grep -q "100.00%"; then
        echo "Methods coverage OK";
    else 
        echo "Insufficient methods coverage";
        error_code=2
    fi
    
    if echo "$output" | grep "Classes:" | head -1 | grep -q "100.00%"; then
        echo "Classes coverage OK";
    else 
        echo "Insufficient classes coverage"; 
        error_code=2
    fi
    
    echo ""
    
    token=$(curl -X POST http://localhost:3000/api/v1/user/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}')
    curl -H "Authorization: Bearer $token" -X POST http://localhost:3000/api/v1/admin/shutdown

    rm -f agdb_server.yaml
    rm -f .agdb_server.agdb
    rm -f agdb_server.agdb
    rm -rf agdb_server_data

    echo ""

    if (( $error_code == 1 )); then
        echo "Tests failed"
    else
        echo "Tests passed"
    fi

    exit $error_code
}

function analyse() {
    ./vendor/bin/phpstan analyse --level=9 -v src tests
}

function format() {
    npx prettier --plugin '@prettier/plugin-php' --write src tests
}

function generate_api() {
    npx @openapitools/openapi-generator-cli generate \
        -i ../../agdb_server/openapi/schema.json \
        -g php \
        -o ./ \
        --additional-properties=invokerPackage=Agdb,artifactVersion=0.7.2
    composer dump-autoload -o
}

function generate_tests() {
    node query_test_generator.js && prettier --plugin '@prettier/plugin-php' --write tests/QueryTest.php
}

if [[ "$1" == "coverage" ]]; then
    coverage
elif [[ "$1" == "analyse" ]]; then
    analyse
elif [[ "$1" == "format" ]]; then
    format
elif [[ "$1" == "format:check" ]]; then
    npx prettier --plugin '@prettier/plugin-php' --check src tests
elif [[ "$1" == "generate" ]]; then
    if [[ "$2" == "api" ]]; then
        generate_api
    elif [[ "$2" == "tests" ]]; then
        generate_tests
    else
        echo "Usage: $0 generate [api|tests]"
        exit 1
    fi
else
    echo "Usage: $0 [coverage|analysis|format|format:check|generate api|generate tests]"
    exit 1
fi
