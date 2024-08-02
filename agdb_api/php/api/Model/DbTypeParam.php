<?php

namespace Agnesoft\Agdb\Model;

class DbTypeParam extends \ArrayObject
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
    protected $dbType;
    /**
     * 
     *
     * @return string
     */
    public function getDbType(): string
    {
        return $this->dbType;
    }
    /**
     * 
     *
     * @param string $dbType
     *
     * @return self
     */
    public function setDbType(string $dbType): self
    {
        $this->initialized['dbType'] = true;
        $this->dbType = $dbType;
        return $this;
    }
}