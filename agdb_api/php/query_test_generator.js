import test_queries from "../../agdb_server/openapi/test_queries.json" assert { type: "json" };
import * as fs from "fs";

let tests = `
// GENERATED. DO NOT MODIFY AS ANY CHANGES WILL BE LOST.
// query_test_generator.js
<?php
use Agnesoft\\Agdb\\QueryBuilder;
use Agnesoft\\Agdb\\DbKeyOrderBuilder;
use Agnesoft\\Agdb\\CountComparisonBuilder;
use Agnesoft\\Agdb\\ComparisonBuilder;

class T { public mixed $db_id = null; public string $value1 = ""; public int $value2 = 0; }

final class QueryTest extends \\PHPUnit\\Framework\\TestCase {
    /** @var array<array<int, mixed>> $test_queries */
    private static $test_queries;

    public static function setUpBeforeClass(): void
    {
        $queries = (array) json_decode(
            (string) file_get_contents(
                "../../agdb_server/openapi/test_queries.json"
            )
        );
        $test_queries = array_map(function ($query) {
            return array($query);
        }, $queries);
        self::$test_queries = $test_queries;
    }`;

for (let index in test_queries) {
    let builder = test_queries[index][0];
    builder = builder.replace(/&/g, "");
    builder = builder.replace(/T::default\(\)/g, "new T()");
    builder = builder.replace(/vec!/g, "");
    builder = builder.replace(/where_/g, "where");
    builder = builder.replace(/\.into\(\)/g, "");
    builder = builder.replace(/\("k","v"\)/g, "\"k\" => \"v\"");
    builder = builder.replace(/\("k",1\)/g, "\"k\" => 1");
    builder = builder.replace(/\("k",2\)/g, "\"k\" => 2");
    builder = builder.replace(/\(1,10\)/g, "1 => 10");
    builder = builder.replace(/\./g, "->");
    builder = builder.replace(/DbKeyOrder/g, "DbKeyOrderBuilder");
    builder = builder.replace(/CountComparison::/g, "CountComparisonBuilder::");
    builder = builder.replace(/Comparison::/g, "ComparisonBuilder::");
    
    tests += `public function testQueryBuilder${index}(): void { $query = ${builder};$json = $query->jsonSerialize();\n$this->assertEquals(self::$test_queries[${index}][1], $json); }\n`;
}

tests += `}`;

fs.writeFileSync("tests/QueryTest.php", tests);
