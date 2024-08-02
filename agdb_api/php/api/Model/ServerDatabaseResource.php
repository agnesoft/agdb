<?php

namespace Agnesoft\Agdb\Model;

class ServerDatabaseResource extends \ArrayObject
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
     * @var string
     */
    protected $resource;
    /**
     * 
     *
     * @return string
     */
    public function getResource(): string
    {
        return $this->resource;
    }
    /**
     * 
     *
     * @param string $resource
     *
     * @return self
     */
    public function setResource(string $resource): self
    {
        $this->initialized['resource'] = true;
        $this->resource = $resource;
        return $this;
    }
}