<?php

namespace Agnesoft\Agdb\Model;

class ClusterStatus extends \ArrayObject
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
    protected $address;
    /**
     * 
     *
     * @var int
     */
    protected $commit;
    /**
     * 
     *
     * @var bool
     */
    protected $leader;
    /**
     * 
     *
     * @var bool
     */
    protected $status;
    /**
     * 
     *
     * @var int
     */
    protected $term;
    /**
     * 
     *
     * @return string
     */
    public function getAddress(): string
    {
        return $this->address;
    }
    /**
     * 
     *
     * @param string $address
     *
     * @return self
     */
    public function setAddress(string $address): self
    {
        $this->initialized['address'] = true;
        $this->address = $address;
        return $this;
    }
    /**
     * 
     *
     * @return int
     */
    public function getCommit(): int
    {
        return $this->commit;
    }
    /**
     * 
     *
     * @param int $commit
     *
     * @return self
     */
    public function setCommit(int $commit): self
    {
        $this->initialized['commit'] = true;
        $this->commit = $commit;
        return $this;
    }
    /**
     * 
     *
     * @return bool
     */
    public function getLeader(): bool
    {
        return $this->leader;
    }
    /**
     * 
     *
     * @param bool $leader
     *
     * @return self
     */
    public function setLeader(bool $leader): self
    {
        $this->initialized['leader'] = true;
        $this->leader = $leader;
        return $this;
    }
    /**
     * 
     *
     * @return bool
     */
    public function getStatus(): bool
    {
        return $this->status;
    }
    /**
     * 
     *
     * @param bool $status
     *
     * @return self
     */
    public function setStatus(bool $status): self
    {
        $this->initialized['status'] = true;
        $this->status = $status;
        return $this;
    }
    /**
     * 
     *
     * @return int
     */
    public function getTerm(): int
    {
        return $this->term;
    }
    /**
     * 
     *
     * @param int $term
     *
     * @return self
     */
    public function setTerm(int $term): self
    {
        $this->initialized['term'] = true;
        $this->term = $term;
        return $this;
    }
}