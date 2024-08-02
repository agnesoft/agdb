<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminUserAdd extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $username;
    /**
     * 
     *
     * @param string $username desired user name
     * @param \Agnesoft\Agdb\Model\UserCredentials $requestBody 
     */
    public function __construct(string $username, \Agnesoft\Agdb\Model\UserCredentials $requestBody)
    {
        $this->username = $username;
        $this->body = $requestBody;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'POST';
    }
    public function getUri(): string
    {
        return str_replace(['{username}'], [$this->username], '/api/v1/admin/user/{username}/add');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        if ($this->body instanceof \Agnesoft\Agdb\Model\UserCredentials) {
            return [['Content-Type' => ['application/json']], $serializer->serialize($this->body, 'json')];
        }
        return [[], null];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddCustom461Exception
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddCustom462Exception
     * @throws \Agnesoft\Agdb\Exception\AdminUserAddCustom463Exception
     *
     * @return null
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (201 === $status) {
            return null;
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminUserAddUnauthorizedException($response);
        }
        if (461 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminUserAddCustom461Exception($response);
        }
        if (462 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminUserAddCustom462Exception($response);
        }
        if (463 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminUserAddCustom463Exception($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}