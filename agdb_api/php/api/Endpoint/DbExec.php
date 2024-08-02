<?php

namespace Agnesoft\Agdb\Endpoint;

class DbExec extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array[] $requestBody 
     */
    public function __construct(string $owner, string $db, array $requestBody)
    {
        $this->owner = $owner;
        $this->db = $db;
        $this->body = $requestBody;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'POST';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/db/{owner}/{db}/exec');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        if (is_array($this->body) and isset($this->body[0]) and is_array($this->body[0])) {
            return [['Content-Type' => ['application/json']], json_encode($this->body)];
        }
        return [[], null];
    }
    public function getExtraHeaders(): array
    {
        return ['Accept' => ['application/json']];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\DbExecUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbExecForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbExecNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\QueryResult[]
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (is_null($contentType) === false && (200 === $status && mb_strpos($contentType, 'application/json') !== false)) {
            return $serializer->deserialize($body, 'Agnesoft\Agdb\Model\QueryResult[]', 'json');
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbExecUnauthorizedException($response);
        }
        if (403 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbExecForbiddenException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbExecNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}