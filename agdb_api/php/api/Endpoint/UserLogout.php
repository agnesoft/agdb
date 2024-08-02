<?php

namespace Agnesoft\Agdb\Endpoint;

class UserLogout extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'POST';
    }
    public function getUri(): string
    {
        return '/api/v1/user/logout';
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\UserLogoutUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\UserLogoutNotFoundException
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
            throw new \Agnesoft\Agdb\Exception\UserLogoutUnauthorizedException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\UserLogoutNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}