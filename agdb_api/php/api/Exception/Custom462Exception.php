<?php

namespace Agnesoft\Agdb\Exception;

class Custom462Exception extends \RuntimeException implements ClientException
{
    public function __construct(string $message)
    {
        parent::__construct($message, 462);
    }
}