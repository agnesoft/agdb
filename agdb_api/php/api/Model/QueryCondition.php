<?php

namespace Agnesoft\Agdb\Model;

class QueryCondition extends \ArrayObject
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
     * Query condition data
     *
     * @var mixed
     */
    protected $data;
    /**
     * Logical operator for query conditions
     *
     * @var string
     */
    protected $logic;
    /**
     * Query condition modifier
     *
     * @var string
     */
    protected $modifier;
    /**
     * Query condition data
     *
     * @return mixed
     */
    public function getData()
    {
        return $this->data;
    }
    /**
     * Query condition data
     *
     * @param mixed $data
     *
     * @return self
     */
    public function setData($data): self
    {
        $this->initialized['data'] = true;
        $this->data = $data;
        return $this;
    }
    /**
     * Logical operator for query conditions
     *
     * @return string
     */
    public function getLogic(): string
    {
        return $this->logic;
    }
    /**
     * Logical operator for query conditions
     *
     * @param string $logic
     *
     * @return self
     */
    public function setLogic(string $logic): self
    {
        $this->initialized['logic'] = true;
        $this->logic = $logic;
        return $this;
    }
    /**
     * Query condition modifier
     *
     * @return string
     */
    public function getModifier(): string
    {
        return $this->modifier;
    }
    /**
     * Query condition modifier
     *
     * @param string $modifier
     *
     * @return self
     */
    public function setModifier(string $modifier): self
    {
        $this->initialized['modifier'] = true;
        $this->modifier = $modifier;
        return $this;
    }
}