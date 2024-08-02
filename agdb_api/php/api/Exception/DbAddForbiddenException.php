<?php

namespace Agnesoft\Agdb\Exception;

class DbAddForbiddenException extends ForbiddenException
{
    /**
     * @var \Psr\Http\Message\ResponseInterface
     */
    private $response;
    public function __construct(\Psr\Http\Message\ResponseInterface $response = null)
    {
        parent::__construct('cannot add db to another user');
        $this->response = $response;
    }
    public function getResponse(): ?\Psr\Http\Message\ResponseInterface
    {
        return $this->response;
    }
}