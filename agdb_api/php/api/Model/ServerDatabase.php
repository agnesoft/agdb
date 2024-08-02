<?php

namespace Agnesoft\Agdb\Model;

class ServerDatabase extends \ArrayObject
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
     * @var int
     */
    protected $backup;
    /**
     * 
     *
     * @var string
     */
    protected $dbType;
    /**
     * 
     *
     * @var string
     */
    protected $name;
    /**
     * 
     *
     * @var string
     */
    protected $role;
    /**
     * 
     *
     * @var int
     */
    protected $size;
    /**
     * 
     *
     * @return int
     */
    public function getBackup(): int
    {
        return $this->backup;
    }
    /**
     * 
     *
     * @param int $backup
     *
     * @return self
     */
    public function setBackup(int $backup): self
    {
        $this->initialized['backup'] = true;
        $this->backup = $backup;
        return $this;
    }
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
    /**
     * 
     *
     * @return string
     */
    public function getName(): string
    {
        return $this->name;
    }
    /**
     * 
     *
     * @param string $name
     *
     * @return self
     */
    public function setName(string $name): self
    {
        $this->initialized['name'] = true;
        $this->name = $name;
        return $this;
    }
    /**
     * 
     *
     * @return string
     */
    public function getRole(): string
    {
        return $this->role;
    }
    /**
     * 
     *
     * @param string $role
     *
     * @return self
     */
    public function setRole(string $role): self
    {
        $this->initialized['role'] = true;
        $this->role = $role;
        return $this;
    }
    /**
     * 
     *
     * @return int
     */
    public function getSize(): int
    {
        return $this->size;
    }
    /**
     * 
     *
     * @param int $size
     *
     * @return self
     */
    public function setSize(int $size): self
    {
        $this->initialized['size'] = true;
        $this->size = $size;
        return $this;
    }
}