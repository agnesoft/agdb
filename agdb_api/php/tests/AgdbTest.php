<?php
use Agnesoft\Agdb\QueryBuilder;
use PHPUnit\Framework\TestCase;

final class AgdbTest extends TestCase
{
    public function testStatus(): void
    {
        $config = Agnesoft\Agdb::Configuration::getDefaultConfiguration();
        $client = new OpenAPI\Client\Api\RoutesApi(new GuzzleHttp\Client(), $config);
        $response = $client->status(false);

        $this->assertIsArray($response);
    }

    public function testQueryBuilder(): void
    {
        $builder = new QueryBuilder();

        $this->assertIsArray($builder->query);
    }
}
