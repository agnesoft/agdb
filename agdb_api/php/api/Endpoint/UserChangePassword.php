<?php

namespace Agnesoft\Agdb\Endpoint;

class UserChangePassword extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    /**
     * 
     *
     * @param \Agnesoft\Agdb\Model\ChangePassword $requestBody 
     */
    public function __construct(\Agnesoft\Agdb\Model\ChangePassword $requestBody)
    {
        $this->body = $requestBody;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'PUT';
    }
    public function getUri(): string
    {
        return '/api/v1/user/change_password';
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        if ($this->body instanceof \Agnesoft\Agdb\Model\ChangePassword) {
            return [['Content-Type' => ['application/json']], $serializer->serialize($this->body, 'json')];
        }
        return [[], null];
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\UserChangePasswordUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\UserChangePasswordCustom461Exception
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
            throw new \Agnesoft\Agdb\Exception\UserChangePasswordUnauthorizedException($response);
        }
        if (461 === $status) {
            throw new \Agnesoft\Agdb\Exception\UserChangePasswordCustom461Exception($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}