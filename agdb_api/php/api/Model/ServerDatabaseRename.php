<?php

namespace Agnesoft\Agdb\Model;

class ServerDatabaseRename extends \ArrayObject
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
    protected $newName;
    /**
     * 
     *
     * @return string
     */
    public function getNewName(): string
    {
        return $this->newName;
    }
    /**
     * 
     *
     * @param string $newName
     *
     * @return self
     */
    public function setNewName(string $newName): self
    {
        $this->initialized['newName'] = true;
        $this->newName = $newName;
        return $this;
    }
}