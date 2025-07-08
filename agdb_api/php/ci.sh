function coverage() {
    rm -f agdb_server.yaml
    rm -rf agdb_server_data

    cargo build -r -p agdb_server
    cargo run -r -p agdb_server &
    
    attempts=0
    max_attempts=10

    echo "Waiting for the server to start..."

    while [ $attempts -lt $max_attempts ]; do
        response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/v1/status 2>/dev/null)
        
        if [ "$response" = "200" ]; then
            echo "Server is ready!"
            break
        fi
        
        attempts=$((attempts + 1))
    done

    local output
    output=$(XDEBUG_MODE=coverage ../../vendor/bin/phpunit tests --coverage-filter src/ --coverage-text --coverage-html coverage/ --coverage-cobertura coverage/coverage-final.xml)
    local error_code=$?
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
    ../../vendor/bin/phpstan analyse --level=9 -v src tests
}

function format() {
    npx prettier --plugin '@prettier/plugin-php' $1 src tests
}

function openapi() {
    rm -rf lib/
    rm -rf docs/

    echo "OSTYPE: $OSTYPE"

    if [[ "$OSTYPE" == "msys" ]]; then
        local package="Agnesoft\AgdbApi"
    else
        local package="Agnesoft\\\\AgdbApi"
    fi

    echo "PACKAGE: $package"
    
    npx @openapitools/openapi-generator-cli generate \
        -i ../../agdb_server/openapi.json \
        -g php \
        -o ./ \
        --additional-properties=invokerPackage=$package,artifactVersion=0.7.2
    
    if [[ "$OSTYPE" == "msys" ]]; then
        for f in $(find lib/ -name '*.*'); do sed -i -e 's~Agnesoft\\\\Agdb~Agnesoft\\Agdb~g' $f; done
        for f in $(find docs/ -name '*.*'); do sed -i -e 's~Agnesoft\\\\Agdb~Agnesoft\\Agdb~g' $f; done
        sed -i -e 's~Agnesoft\\\\Agdb~Agnesoft\\Agdb~g' README.md
    fi

    echo "composer dump-autoload..."
    composer --version
    cd ../../
    composer dump-autoload -o
}

function test_queries() {
    node query_test_generator.js && npx prettier --plugin '@prettier/plugin-php' --write tests/QueryTest.php
}

if [[ "$1" == "coverage" ]]; then
    coverage
elif [[ "$1" == "analyse" ]]; then
    analyse
elif [[ "$1" == "format" ]]; then
    format "--write"
elif [[ "$1" == "format:check" ]]; then
    format "--check"
elif [[ "$1" == "openapi" ]]; then
    openapi
elif [[ "$1" == "test_queries" ]]; then
    test_queries
else
    echo "Usage: $0 [coverage|analyse|format|format:check|openapi|test_queries]"
    exit 1
fi
