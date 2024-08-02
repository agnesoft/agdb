<?php

namespace Agnesoft\Agdb\Model;

class InsertNodesQuery extends \ArrayObject
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
     * Aliases of the new nodes.
     *
     * @var list<string>
     */
    protected $aliases;
    /**
     * Number of nodes to be inserted.
     *
     * @var int
     */
    protected $count;
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
    * Helper type distinguishing uniform (`Single`) values
    and multiple (`Multi`) values in database queries.
    *
    * @var mixed
    */
    protected $values;
    /**
     * Aliases of the new nodes.
     *
     * @return list<string>
     */
    public function getAliases(): array
    {
        return $this->aliases;
    }
    /**
     * Aliases of the new nodes.
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
     * Number of nodes to be inserted.
     *
     * @return int
     */
    public function getCount(): int
    {
        return $this->count;
    }
    /**
     * Number of nodes to be inserted.
     *
     * @param int $count
     *
     * @return self
     */
    public function setCount(int $count): self
    {
        $this->initialized['count'] = true;
        $this->count = $count;
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
    /**
    * Helper type distinguishing uniform (`Single`) values
    and multiple (`Multi`) values in database queries.
    *
    * @return mixed
    */
    public function getValues()
    {
        return $this->values;
    }
    /**
    * Helper type distinguishing uniform (`Single`) values
    and multiple (`Multi`) values in database queries.
    *
    * @param mixed $values
    *
    * @return self
    */
    public function setValues($values): self
    {
        $this->initialized['values'] = true;
        $this->values = $values;
        return $this;
    }
}