<?php

namespace Agnesoft\Agdb\Endpoint;

class DbRename extends \Agnesoft\Agdb\Runtime\Client\BaseEndpoint implements \Agnesoft\Agdb\Runtime\Client\Endpoint
{
    protected $owner;
    protected $db;
    /**
     * 
     *
     * @param string $owner db owner user name
     * @param string $db db name
     * @param array $queryParameters {
     *     @var string $new_name 
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
        return str_replace(['{owner}', '{db}'], [$this->owner, $this->db], '/api/v1/db/{owner}/{db}/rename');
    }
    public function getBody(\Symfony\Component\Serializer\SerializerInterface $serializer, $streamFactory = null): array
    {
        return [[], null];
    }
    protected function getQueryOptionsResolver(): \Symfony\Component\OptionsResolver\OptionsResolver
    {
        $optionsResolver = parent::getQueryOptionsResolver();
        $optionsResolver->setDefined(['new_name']);
        $optionsResolver->setRequired(['new_name']);
        $optionsResolver->setDefaults([]);
        $optionsResolver->addAllowedTypes('new_name', ['string']);
        return $optionsResolver;
    }
    /**
     * {@inheritdoc}
     *
     * @throws \Agnesoft\Agdb\Exception\DbRenameUnauthorizedException
     * @throws \Agnesoft\Agdb\Exception\DbRenameForbiddenException
     * @throws \Agnesoft\Agdb\Exception\DbRenameNotFoundException
     * @throws \Agnesoft\Agdb\Exception\DbRenameCustom465Exception
     * @throws \Agnesoft\Agdb\Exception\DbRenameCustom467Exception
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
            throw new \Agnesoft\Agdb\Exception\DbRenameUnauthorizedException($response);
        }
        if (403 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbRenameForbiddenException($response);
        }
        if (404 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbRenameNotFoundException($response);
        }
        if (465 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbRenameCustom465Exception($response);
        }
        if (467 === $status) {
            throw new \Agnesoft\Agdb\Exception\DbRenameCustom467Exception($response);
        }
    }
    public function getAuthenticationScopes(): array
    {
        return ['Token'];
    }
}