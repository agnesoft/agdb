<?php

namespace Agnesoft\Agdb\Model;

class QueryResult extends \ArrayObject
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
    * List of elements yielded by the query
    possibly with a list of properties.
    *
    * @var list<DbElement>
    */
    protected $elements;
    /**
     * Query result
     *
     * @var int
     */
    protected $result;
    /**
    * List of elements yielded by the query
    possibly with a list of properties.
    *
    * @return list<DbElement>
    */
    public function getElements(): array
    {
        return $this->elements;
    }
    /**
    * List of elements yielded by the query
    possibly with a list of properties.
    *
    * @param list<DbElement> $elements
    *
    * @return self
    */
    public function setElements(array $elements): self
    {
        $this->initialized['elements'] = true;
        $this->elements = $elements;
        return $this;
    }
    /**
     * Query result
     *
     * @return int
     */
    public function getResult(): int
    {
        return $this->result;
    }
    /**
     * Query result
     *
     * @param int $result
     *
     * @return self
     */
    public function setResult(int $result): self
    {
        $this->initialized['result'] = true;
        $this->result = $result;
        return $this;
    }
}