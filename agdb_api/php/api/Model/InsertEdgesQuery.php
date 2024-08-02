<?php

namespace Agnesoft\Agdb\Model;

class InsertEdgesQuery extends \ArrayObject
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
    * If `true` create an edge between each origin
    and destination.
    *
    * @var bool
    */
    protected $each;
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
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @var mixed
    */
    protected $to;
    /**
    * Helper type distinguishing uniform (`Single`) values
    and multiple (`Multi`) values in database queries.
    *
    * @var mixed
    */
    protected $values;
    /**
    * If `true` create an edge between each origin
    and destination.
    *
    * @return bool
    */
    public function getEach(): bool
    {
        return $this->each;
    }
    /**
    * If `true` create an edge between each origin
    and destination.
    *
    * @param bool $each
    *
    * @return self
    */
    public function setEach(bool $each): self
    {
        $this->initialized['each'] = true;
        $this->each = $each;
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
    public function getFrom()
    {
        return $this->from;
    }
    /**
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @param mixed $from
    *
    * @return self
    */
    public function setFrom($from): self
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
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @return mixed
    */
    public function getTo()
    {
        return $this->to;
    }
    /**
    * List of database ids used in queries. It
    can either represent a list of [`QueryId`]s
    or a search query. Search query allows query
    nesting and sourcing the ids dynamically for
    another query most commonly with the
    select queries.
    *
    * @param mixed $to
    *
    * @return self
    */
    public function setTo($to): self
    {
        $this->initialized['to'] = true;
        $this->to = $to;
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