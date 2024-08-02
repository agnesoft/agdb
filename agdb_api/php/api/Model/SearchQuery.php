<?php

namespace Agnesoft\Agdb\Model;

class SearchQuery extends \ArrayObject
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
     * Search algorithm to be used
     *
     * @var string
     */
    protected $algorithm;
    /**
    * Set of conditions every element must satisfy to be included in the
    result. Some conditions also influence the search path as well.
    *
    * @var list<QueryCondition>
    */
    protected $conditions;
    /**
    * Database id used in queries that lets
    you refer to a database element as numerical
    id or a string alias.
    *
    * @var mixed
    */
    protected $destination;
    /**
     * How many elements maximum to return.
     *
     * @var int
     */
    protected $limit;
    /**
    * How many elements that would be returned should be
    skipped in the result.
    *
    * @var int
    */
    protected $offset;
    /**
    * Order of the elements in the result. The sorting happens before
    `offset` and `limit` are applied.
    *
    * @var list<mixed>
    */
    protected $orderBy;
    /**
    * Database id used in queries that lets
    you refer to a database element as numerical
    id or a string alias.
    *
    * @var mixed
    */
    protected $origin;
    /**
     * Search algorithm to be used
     *
     * @return string
     */
    public function getAlgorithm(): string
    {
        return $this->algorithm;
    }
    /**
     * Search algorithm to be used
     *
     * @param string $algorithm
     *
     * @return self
     */
    public function setAlgorithm(string $algorithm): self
    {
        $this->initialized['algorithm'] = true;
        $this->algorithm = $algorithm;
        return $this;
    }
    /**
    * Set of conditions every element must satisfy to be included in the
    result. Some conditions also influence the search path as well.
    *
    * @return list<QueryCondition>
    */
    public function getConditions(): array
    {
        return $this->conditions;
    }
    /**
    * Set of conditions every element must satisfy to be included in the
    result. Some conditions also influence the search path as well.
    *
    * @param list<QueryCondition> $conditions
    *
    * @return self
    */
    public function setConditions(array $conditions): self
    {
        $this->initialized['conditions'] = true;
        $this->conditions = $conditions;
        return $this;
    }
    /**
    * Database id used in queries that lets
    you refer to a database element as numerical
    id or a string alias.
    *
    * @return mixed
    */
    public function getDestination()
    {
        return $this->destination;
    }
    /**
    * Database id used in queries that lets
    you refer to a database element as numerical
    id or a string alias.
    *
    * @param mixed $destination
    *
    * @return self
    */
    public function setDestination($destination): self
    {
        $this->initialized['destination'] = true;
        $this->destination = $destination;
        return $this;
    }
    /**
     * How many elements maximum to return.
     *
     * @return int
     */
    public function getLimit(): int
    {
        return $this->limit;
    }
    /**
     * How many elements maximum to return.
     *
     * @param int $limit
     *
     * @return self
     */
    public function setLimit(int $limit): self
    {
        $this->initialized['limit'] = true;
        $this->limit = $limit;
        return $this;
    }
    /**
    * How many elements that would be returned should be
    skipped in the result.
    *
    * @return int
    */
    public function getOffset(): int
    {
        return $this->offset;
    }
    /**
    * How many elements that would be returned should be
    skipped in the result.
    *
    * @param int $offset
    *
    * @return self
    */
    public function setOffset(int $offset): self
    {
        $this->initialized['offset'] = true;
        $this->offset = $offset;
        return $this;
    }
    /**
    * Order of the elements in the result. The sorting happens before
    `offset` and `limit` are applied.
    *
    * @return list<mixed>
    */
    public function getOrderBy(): array
    {
        return $this->orderBy;
    }
    /**
    * Order of the elements in the result. The sorting happens before
    `offset` and `limit` are applied.
    *
    * @param list<mixed> $orderBy
    *
    * @return self
    */
    public function setOrderBy(array $orderBy): self
    {
        $this->initialized['orderBy'] = true;
        $this->orderBy = $orderBy;
        return $this;
    }
    /**
    * Database id used in queries that lets
    you refer to a database element as numerical
    id or a string alias.
    *
    * @return mixed
    */
    public function getOrigin()
    {
        return $this->origin;
    }
    /**
    * Database id used in queries that lets
    you refer to a database element as numerical
    id or a string alias.
    *
    * @param mixed $origin
    *
    * @return self
    */
    public function setOrigin($origin): self
    {
        $this->initialized['origin'] = true;
        $this->origin = $origin;
        return $this;
    }
}