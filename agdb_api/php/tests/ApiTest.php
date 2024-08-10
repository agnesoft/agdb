<?php
use PHPUnit\Framework\TestCase;


final class ApiTest extends TestCase
{
    public function testStatus(): void
    {
        $config = Agdb\Configuration::getDefaultConfiguration();
        $client = new Agdb\Api\RoutesApi(new GuzzleHttp\Client(), $config);
        $response = $client->status(false);

        $this->assertIsArray($response);
    }
}
