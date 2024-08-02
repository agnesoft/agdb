<?php

namespace Agnesoft\Agdb\Model;

class InsertAliasesQuery extends \ArrayObject
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
     * Aliases to be inserted
     *
     * @var list<string>
     */
    protected $aliases;
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
     * Aliases to be inserted
     *
     * @return list<string>
     */
    public function getAliases(): array
    {
        return $this->aliases;
    }
    /**
     * Aliases to be inserted
     *
     * @param list<string> $aliases
     *
     * @return self
     */
    public function setAliases(array $aliases): self
    {
        $this->initialized['aliases'] = true;
        $this->aliases = $aliases;
        return $this;
    }
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
}