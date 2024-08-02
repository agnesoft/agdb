<?php

namespace Agnesoft\Agdb\Model;

class QueryAudit extends \ArrayObject
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
     * Convenience enum for serializing/deserializing queries.
     *
     * @var mixed
     */
    protected $query;
    /**
     * 
     *
     * @var int
     */
    protected $timestamp;
    /**
     * 
     *
     * @var string
     */
    protected $user;
    /**
     * Convenience enum for serializing/deserializing queries.
     *
     * @return mixed
     */
    public function getQuery()
    {
        return $this->query;
    }
    /**
     * Convenience enum for serializing/deserializing queries.
     *
     * @param mixed $query
     *
     * @return self
     */
    public function setQuery($query): self
    {
        $this->initialized['query'] = true;
        $this->query = $query;
        return $this;
    }
    /**
     * 
     *
     * @return int
     */
    public function getTimestamp(): int
    {
        return $this->timestamp;
    }
    /**
     * 
     *
     * @param int $timestamp
     *
     * @return self
     */
    public function setTimestamp(int $timestamp): self
    {
        $this->initialized['timestamp'] = true;
        $this->timestamp = $timestamp;
        return $this;
    }
    /**
     * 
     *
     * @return string
     */
    public function getUser(): string
    {
        return $this->user;
    }
    /**
     * 
     *
     * @param string $user
     *
     * @return self
     */
    public function setUser(string $user): self
    {
        $this->initialized['user'] = true;
        $this->user = $user;
        return $this;
    }
}