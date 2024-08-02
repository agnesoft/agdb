<?php
use Agnesoft\Agdb\QueryBuilder;
use PHPUnit\Framework\TestCase;
use Agnesoft\Agdb\Client;

final class AgdbTest extends TestCase
{
    public function testStatus(): void
    {
        $client = Client::create();
        $response = $client->status(false, 'response');

        $this->assertEquals(200, $response->getStatusCode());
    }

    public function testQueryBuilder(): void
    {
        $client = new QueryBuilder();
    }
}
