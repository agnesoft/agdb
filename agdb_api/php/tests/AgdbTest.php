<?php
use PHPUnit\Framework\TestCase;
use Agnesoft\Agdb\Client;

final class AgdbTest extends TestCase
{
    public function testAgdb(): void
    {
        $client = Client::create();
        $response = $client->status(false, 'response');

        $this->assertEquals(200, $response->getStatusCode());
    }
}
