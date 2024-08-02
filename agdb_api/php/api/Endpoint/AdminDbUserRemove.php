<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminDbUserRemove extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    protected $username;
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param string $username user name
     */
    public function __construct(string $owner, string $db, string $username)
    {
        $this->owner = $owner;
        $this->db = $db;
        $this->username = $username;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'DELETE';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}', '{username}'], [$this->owner, $this->db, $this->username], '/api/v1/admin/db/{owner}/{db}/user/{username}/remove');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserRemoveForbiddenException
     * @throws \Agnesoft\Agdb\Exception\AdminDbUserRemoveNotFoundException
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
            throw new \Agnesoft\Agdb\Exception\AdminDbUserRemoveUnauthorizedException($response);
        }
        if (403 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbUserRemoveForbiddenException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbUserRemoveNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}