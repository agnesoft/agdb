function coverage() {
    local output=$(XDEBUG_MODE=coverage ./vendor/bin/phpunit tests --coverage-filter src/ --coverage-text --coverage-html coverage/)
    echo "$output"
    echo ""

    if echo "$output" | grep "Lines:" | head -1 | grep -q "100.00%"; then
        echo "Line coverage OK";
    else
        echo "Insufficient line coverage";
        exit 1;
    fi
    
    if echo "$output" | grep "Methods:" | head -1 | grep -q "100.00%"; then
        echo "Methods coverage OK";
    else 
        echo "Insufficient methods coverage";
        exit 1;
    fi
    
    if echo "$output" | grep "Classes:" | head -1 | grep -q "100.00%"; then
        echo "Classes coverage OK";
    else 
        echo "Insufficient classes coverage"; 
        exit 1;
    fi
}

function analysis() {
    ./vendor/bin/phpstan analyse --level=9 src tests
}

function format() {
    npx prettier --plugin '@prettier/plugin-php' --write src tests
}

if [[ "$1" == "coverage" ]]; then
    coverage
elif [[ "$1" == "analysis" ]]; then
    analysis
elif [[ "$1" == "format" ]]; then
    format
elif [[ "$1" == "format:check" ]]; then
    npx prettier --plugin '@prettier/plugin-php' --check src tests
fi
