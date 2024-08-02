<?php

namespace Agnesoft\Agdb\Exception;

class Custom464Exception extends \RuntimeException implements ClientException
{
    public function __construct(string $message)
    {
        parent::__construct($message, 464);
    }
}