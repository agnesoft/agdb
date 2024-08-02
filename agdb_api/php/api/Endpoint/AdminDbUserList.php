<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminDbUserList extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    /**
     * 
     *
     * @param string $owner db owner user name
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
        return 'GET';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/admin/db/{owner}/{db}/user/list');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    public function getExtraHeaders(): array
    {
        return ['Accept' => ['application/json']];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserListUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserListNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\DbUser[]
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (is_null($contentType) === false && (200 === $status && mb_strpos($contentType, 'application/json') !== false)) {
            return $serializer->deserialize($body, 'Agnesoft\Agdb\Model\DbUser[]', 'json');
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbUserListUnauthorizedException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbUserListNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}