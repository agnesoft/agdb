<?php

namespace Agnesoft\Agdb\Model;

class InsertValuesQuery extends \ArrayObject
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
    * Helper type distinguishing uniform (`Single`) values
    and multiple (`Multi`) values in database queries.
    *
    * @var mixed
    */
    protected $values;
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