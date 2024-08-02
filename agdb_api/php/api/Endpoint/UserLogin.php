<?php

namespace Agnesoft\Agdb\Endpoint;

class UserLogin extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    /**
     * 
     *
     * @param \Agnesoft\Agdb\Model\UserLogin $requestBody 
     */
    public function __construct(\Agnesoft\Agdb\Model\UserLogin $requestBody)
    {
        $this->body = $requestBody;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'POST';
    }
    public function getUri(): string
    {
        return '/api/v1/user/login';
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        if ($this->body instanceof \Agnesoft\Agdb\Model\UserLogin) {
            return [['Content-Type' => ['application/json']], $serializer->serialize($this->body, 'json')];
        }
        return [[], null];
    }
    public function getExtraHeaders(): array
    {
        return ['Accept' => ['text/plain']];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\UserLoginUnauthorizedException
     *
     * @return null
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (200 === $status) {
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\UserLoginUnauthorizedException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return [];
    }
}