<?php
use Agnesoft\Agdb\QueryBuilder;
use PHPUnit\Framework\TestCase;



final class AgdbTest extends TestCase
{
    // public function testStatus(): void
    // {
    //     $config = Agdb\Configuration::getDefaultConfiguration();
    //     $client = new Agdb\Api\RoutesApi(new GuzzleHttp\Client(), $config);
    //     $response = $client->status(false);

    //     $this->assertIsArray($response);
    // }

    // private static $test_queries;

    // public static function setUpBeforeClass(): void
    // {
    //     self::$test_queries = json_decode(
    //         file_get_contents("../../agdb_server/openapi/test_queries.json")
    //     );
    // }

    // public function testQueryBuilder0(): void
    // {
    //     $query = QueryBuilder::insert()->aliases("a")->ids(1)->query();
    //     $json = json_encode($query->jsonSerialize());
    //     $this->assertJsonStringEqualsJsonString($json, json_encode(self::$test_queries[0][1]));
    // }
}
