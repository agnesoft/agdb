<?php

namespace Agnesoft\Agdb\Model;

class StatusParams extends \ArrayObject
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
     * @var bool|null
     */
    protected $cluster;
    /**
     * 
     *
     * @return bool|null
     */
    public function getCluster(): ?bool
    {
        return $this->cluster;
    }
    /**
     * 
     *
     * @param bool|null $cluster
     *
     * @return self
     */
    public function setCluster(?bool $cluster): self
    {
        $this->initialized['cluster'] = true;
        $this->cluster = $cluster;
        return $this;
    }
}