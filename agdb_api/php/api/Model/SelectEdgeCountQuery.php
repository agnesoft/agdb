<?php

namespace Agnesoft\Agdb\Model;

class SelectEdgeCountQuery extends \ArrayObject
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
    * If set to `true` the query will count outgoing edges
    from the nodes.
    *
    * @var bool
    */
    protected $from;
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
    * If set to `true` the query will count incoming edges
    to the nodes.
    *
    * @var bool
    */
    protected $to;
    /**
    * If set to `true` the query will count outgoing edges
    from the nodes.
    *
    * @return bool
    */
    public function getFrom(): bool
    {
        return $this->from;
    }
    /**
    * If set to `true` the query will count outgoing edges
    from the nodes.
    *
    * @param bool $from
    *
    * @return self
    */
    public function setFrom(bool $from): self
    {
        $this->initialized['from'] = true;
        $this->from = $from;
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
    * If set to `true` the query will count incoming edges
    to the nodes.
    *
    * @return bool
    */
    public function getTo(): bool
    {
        return $this->to;
    }
    /**
    * If set to `true` the query will count incoming edges
    to the nodes.
    *
    * @param bool $to
    *
    * @return self
    */
    public function setTo(bool $to): self
    {
        $this->initialized['to'] = true;
        $this->to = $to;
        return $this;
    }
}