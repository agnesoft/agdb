function coverage() {
    cd ../../agdb_server
    rm -f agdb_server.yaml
    rm -f .agdb_server.agdb
    rm -f agdb_server.agdb
    rm -rf agdb_server_data
    cargo build --release
    cargo run --release &
    
    cd ../agdb_api/php
    local output=$(XDEBUG_MODE=coverage ./vendor/bin/phpunit tests --coverage-filter src/ --coverage-text --coverage-html coverage/)
    local error_code=$?   
    echo "$output"
    echo ""

    if echo "$output" | grep "Lines:" | head -1 | grep -q "100.00%"; then
        echo "Line coverage OK";
    else
        echo "Insufficient line coverage";
        error_code=1
    fi
    
    if echo "$output" | grep "Methods:" | head -1 | grep -q "100.00%"; then
        echo "Methods coverage OK";
    else 
        echo "Insufficient methods coverage";
        error_code=1
    fi
    
    if echo "$output" | grep "Classes:" | head -1 | grep -q "100.00%"; then
        echo "Classes coverage OK";
    else 
        echo "Insufficient classes coverage"; 
        error_code=1
    fi

    token=$(curl -X POST http://localhost:3000/api/v1/user/login -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}')
    curl -H "Authorization: Bearer $token" -X POST http://localhost:3000/api/v1/admin/shutdown

    cd ../../agdb_server
    rm -f agdb_server.yaml
    rm -f .agdb_server.agdb
    rm -f agdb_server.agdb
    rm -rf agdb_server_data

    exit $error_code
}

function analysis() {
    ./vendor/bin/phpstan analyse --level=9 src tests
}

function format() {
    npx prettier --plugin '@prettier/plugin-php' --write src tests
}

function generate_api() {
    php vendor/bin/jane-openapi generate
    sed -i -e 's/localhost/localhost:3000/' ./api/Client.php
    composer dump-autoload -o
}

function generate_tests() {
    echo "TODO"
}

if [[ "$1" == "coverage" ]]; then
    coverage
elif [[ "$1" == "analysis" ]]; then
    analysis
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
