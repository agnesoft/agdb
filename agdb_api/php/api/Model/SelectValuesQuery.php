<?php

namespace Agnesoft\Agdb\Model;

class SelectValuesQuery extends \ArrayObject
{
    /**
     * @var array
     */
    protected $initialized = [];
    public function isInitialized($property): bool
    {
        return array_key_exists($property, $this->initialized);
    }
    /**
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @var mixed
    */
    protected $ids;
    /**
     * 
     *
     * @var list<mixed>
     */
    protected $keys;
    /**
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @return mixed
    */
    public function getIds()
    {
        return $this->ids;
    }
    /**
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @param mixed $ids
    *
    * @return self
    */
    public function setIds($ids): self
    {
        $this->initialized['ids'] = true;
        $this->ids = $ids;
        return $this;
    }
    /**
     * 
     *
     * @return list<mixed>
     */
    public function getKeys(): array
    {
        return $this->keys;
    }
    /**
     * 
     *
     * @param list<mixed> $keys
     *
     * @return self
     */
    public function setKeys(array $keys): self
    {
        $this->initialized['keys'] = true;
        $this->keys = $keys;
        return $this;
    }
}