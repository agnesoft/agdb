<?php

namespace Agnesoft\Agdb\Endpoint;

class DbUserAdd extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
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
     * @param array $queryParameters {
     *     @var string $db_role 
     * }
     */
    public function __construct(string $owner, string $db, string $username, array $queryParameters = [])
    {
        $this->owner = $owner;
        $this->db = $db;
        $this->username = $username;
        $this->queryParameters = $queryParameters;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'POST';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}', '{username}'], [$this->owner, $this->db, $this->username], '/api/v1/db/{owner}/{db}/user/{username}/add');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    protected function getQueryOptionsResolver(): \Symfony\Component\OptionsResolver\OptionsResolver
    {
        $optionsResolver = parent::getQueryOptionsResolver();
        $optionsResolver->setDefined(['db_role']);
        $optionsResolver->setRequired(['db_role']);
        $optionsResolver->setDefaults([]);
        $optionsResolver->addAllowedTypes('db_role', ['string']);
        return $optionsResolver;
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\DbUserAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbUserAddForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbUserAddNotFoundException
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
            throw new \Agnesoft\Agdb\Exception\DbUserAddUnauthorizedException($response);
        }
        if (403 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbUserAddForbiddenException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbUserAddNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}