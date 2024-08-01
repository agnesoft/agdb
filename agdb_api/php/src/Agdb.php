<?php declare(strict_types=1);
namespace agdb;

class QueryBuilder
{
    public string $query;

    function __construct()
    {
        $this->query = "";
    }
}
