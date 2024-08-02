<?php

namespace Agnesoft\Agdb\Endpoint;

class DbClear extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    /**
     * 
     *
     * @param string $owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $resource 
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
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/db/{owner}/{db}/clear');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    public function getExtraHeaders(): array
    {
        return ['Accept' => ['application/json']];
    }
    protected function getQueryOptionsResolver(): \Symfony\Component\OptionsResolver\OptionsResolver
    {
        $optionsResolver = parent::getQueryOptionsResolver();
        $optionsResolver->setDefined(['resource']);
        $optionsResolver->setRequired(['resource']);
        $optionsResolver->setDefaults([]);
        $optionsResolver->addAllowedTypes('resource', ['string']);
        return $optionsResolver;
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\DbClearUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbClearForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbClearNotFoundException
     *
     * @return null|\Agnesoft\Agdb\Model\ServerDatabase
     */
    protected function transformResponseBody(\Psr\Http\Message\ResponseInterface $response, \Symfony\Component\Serializer\SerializerInterface $serializer, ?string $contentType = null)
    {
        $status = $response->getStatusCode();
        $body = (string) $response->getBody();
        if (is_null($contentType) === false && (201 === $status && mb_strpos($contentType, 'application/json') !== false)) {
            return $serializer->deserialize($body, 'Agnesoft\Agdb\Model\ServerDatabase', 'json');
        }
        if (401 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbClearUnauthorizedException($response);
        }
        if (403 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbClearForbiddenException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbClearNotFoundException($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}