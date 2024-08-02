<?php

namespace Agnesoft\Agdb\Endpoint;

class AdminDbAdd extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $db_type 
     * }
     */
    public function __construct(string $owner, string $db, array $queryParameters = [])
    {
        $this->owner = $owner;
        $this->db = $db;
        $this->queryParameters = $queryParameters;
    }
    use \Agnesoft\Agdb\Runtime\Client\EndpointTrait;
    public function getMethod(): string
    {
        return 'POST';
    }
    public function getUri(): string
    {
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/admin/db/{owner}/{db}/add');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    protected function getQueryOptionsResolver(): \Symfony\Component\OptionsResolver\OptionsResolver
    {
        $optionsResolver = parent::getQueryOptionsResolver();
        $optionsResolver->setDefined(['db_type']);
        $optionsResolver->setRequired(['db_type']);
        $optionsResolver->setDefaults([]);
        $optionsResolver->addAllowedTypes('db_type', ['string']);
        return $optionsResolver;
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\AdminDbAddUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\AdminDbAddNotFoundException
     * @throws \Agnesoft\Agdb\Exception\AdminDbAddCustom465Exception
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
            throw new \Agnesoft\Agdb\Exception\AdminDbAddUnauthorizedException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbAddNotFoundException($response);
        }
        if (465 === $status) {
            throw new \Agnesoft\Agdb\Exception\AdminDbAddCustom465Exception($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}