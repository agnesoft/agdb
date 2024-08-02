<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminDbDelete extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     */
    public function __construct(string $owner, string $db)
    {
        $this->owner = $owner;
        $this->db = $db;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'DELETE';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/admin/db/{owner}/{db}/delete');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\AdminDbDeleteUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbDeleteNotFoundException
     *
     * @return null
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (204 === $status) {
            return null;
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbDeleteUnauthorizedException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbDeleteNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}