<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminDbAudit extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
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
        return 'GET';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/admin/db/{owner}/{db}/audit');
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
     * @throws \Agnesoft\Agdb\Exception\AdminDbAuditUnauthorizedException
     *
     * @return null|\Agnesoft\Agdb\Model\QueryAudit[]
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (is_null($contentType) === false && (200 === $status && mb_strpos($contentType, 'application/json') !== false)) {
            return $serializer->deserialize($body, 'Agnesoft\Agdb\Model\QueryAudit[]', 'json');
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbAuditUnauthorizedException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}