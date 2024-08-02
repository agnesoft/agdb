<?php

namespace Agnesoft\Agdb\Exception;

class Custom465Exception extends \RuntimeException implements ClientException
{
    public function __construct(string $message)
    {
        parent::__construct($message, 465);
    }
}