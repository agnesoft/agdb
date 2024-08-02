<?php

namespace Agnesoft\Agdb\Model;

class DbElement extends \ArrayObject
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
     * 
     *
     * @var int|null
     */
    protected $from;
    /**
    * Database id is a wrapper around `i64`.
    The id is an identifier of a database element
    both nodes and edges. The positive ids represent nodes,
    negative ids represent edges. The value of `0` is
    logically invalid (there cannot be element with id 0) and a default.
    *
    * @var int
    */
    protected $id;
    /**
     * 
     *
     * @var int|null
     */
    protected $to;
    /**
     * List of key-value pairs associated with the element.
     *
     * @var list<DbKeyValue>
     */
    protected $values;
    /**
     * 
     *
     * @return int|null
     */
    public function getFrom(): ?int
    {
        return $this->from;
    }
    /**
     * 
     *
     * @param int|null $from
     *
     * @return self
     */
    public function setFrom(?int $from): self
    {
        $this->initialized['from'] = true;
        $this->from = $from;
        return $this;
    }
    /**
    * Database id is a wrapper around `i64`.
    The id is an identifier of a database element
    both nodes and edges. The positive ids represent nodes,
    negative ids represent edges. The value of `0` is
    logically invalid (there cannot be element with id 0) and a default.
    *
    * @return int
    */
    public function getId(): int
    {
        return $this->id;
    }
    /**
    * Database id is a wrapper around `i64`.
    The id is an identifier of a database element
    both nodes and edges. The positive ids represent nodes,
    negative ids represent edges. The value of `0` is
    logically invalid (there cannot be element with id 0) and a default.
    *
    * @param int $id
    *
    * @return self
    */
    public function setId(int $id): self
    {
        $this->initialized['id'] = true;
        $this->id = $id;
        return $this;
    }
    /**
     * 
     *
     * @return int|null
     */
    public function getTo(): ?int
    {
        return $this->to;
    }
    /**
     * 
     *
     * @param int|null $to
     *
     * @return self
     */
    public function setTo(?int $to): self
    {
        $this->initialized['to'] = true;
        $this->to = $to;
        return $this;
    }
    /**
     * List of key-value pairs associated with the element.
     *
     * @return list<DbKeyValue>
     */
    public function getValues(): array
    {
        return $this->values;
    }
    /**
     * List of key-value pairs associated with the element.
     *
     * @param list<DbKeyValue> $values
     *
     * @return self
     */
    public function setValues(array $values): self
    {
        $this->initialized['values'] = true;
        $this->values = $values;
        return $this;
    }
}