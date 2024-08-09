<?php
use Agnesoft\Agdb\QueryBuilder;
use PHPUnit\Framework\TestCase;
use Agnesoft\Agdb\DbKeyOrderBuilder;


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

    // public function testQueryBuilder108(): void
    // {
    //     $query = QueryBuilder::search()
    //         ->to(1)
    //         ->order_by([DbKeyOrderBuilder::Asc("k")])
    //         ->where()
    //         ->node()
    //         ->query();
    //     $json = $query->jsonSerialize();
    //     print (json_encode($json));
    //     $this->assertEquals(self::$test_queries[108][1], $json);
    // }
}
