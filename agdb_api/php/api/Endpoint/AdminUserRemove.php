<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminUserRemove extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $username;
    /**
     * 
     *
     * @param string $username user name
     */
    public function __construct(string $username)
    {
        $this->username = $username;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'DELETE';
    }
    public function getUri(): string
    {
        return str_replace(['{username}'], [$this->username], '/api/v1/admin/user/{username}/remove');
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
     * @throws \Agnesoft\Agdb\Exception\AdminUserRemoveUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminUserRemoveNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\UserStatus[]
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (is_null($contentType) === false && (204 === $status && mb_strpos($contentType, 'application/json') !== false)) {
            return $serializer->deserialize($body, 'Agnesoft\Agdb\Model\UserStatus[]', 'json');
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminUserRemoveUnauthorizedException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminUserRemoveNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}