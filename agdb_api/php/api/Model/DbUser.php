<?php

namespace Agnesoft\Agdb\Model;

class DbUser extends \ArrayObject
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
    protected $role;
    /**
     * 
     *
     * @var string
     */
    protected $user;
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