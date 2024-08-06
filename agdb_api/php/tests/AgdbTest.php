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

    public function testQueryBuilder(): void
    {
        $query = QueryBuilder::insert()->aliases('alias')->ids([10, 20])->query();
        $json = json_encode($query->jsonSerialize());

        echo $json;
        $this->assertIsString($json);
        $this->assertSame(true, false);

    }
}
