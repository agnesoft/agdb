<?php

namespace Agnesoft\Agdb\Exception;

class Custom463Exception extends \RuntimeException implements ClientException
{
    public function __construct(string $message)
    {
        parent::__construct($message, 463);
    }
}