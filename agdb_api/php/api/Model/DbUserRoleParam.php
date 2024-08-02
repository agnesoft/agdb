<?php

namespace Agnesoft\Agdb\Model;

class DbUserRoleParam extends \ArrayObject
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
    protected $dbRole;
    /**
     * 
     *
     * @return string
     */
    public function getDbRole(): string
    {
        return $this->dbRole;
    }
    /**
     * 
     *
     * @param string $dbRole
     *
     * @return self
     */
    public function setDbRole(string $dbRole): self
    {
        $this->initialized['dbRole'] = true;
        $this->dbRole = $dbRole;
        return $this;
    }
}