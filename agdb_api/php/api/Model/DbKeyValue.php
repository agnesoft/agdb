<?php

namespace Agnesoft\Agdb\Model;

class DbKeyValue extends \ArrayObject
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
    * Database value is a strongly types value.
    
    It is an enum of limited number supported types
    that are universal across all platforms
    and programming languages.
    
    The value is constructible from large number of
    raw types or associated types (e.g. i32, &str, etc.).
    Getting the raw value back as string can be done
    with `to_string()` but otherwise requires a `match`.
    *
    * @var mixed
    */
    protected $key;
    /**
    * Database value is a strongly types value.
    
    It is an enum of limited number supported types
    that are universal across all platforms
    and programming languages.
    
    The value is constructible from large number of
    raw types or associated types (e.g. i32, &str, etc.).
    Getting the raw value back as string can be done
    with `to_string()` but otherwise requires a `match`.
    *
    * @var mixed
    */
    protected $value;
    /**
    * Database value is a strongly types value.
    
    It is an enum of limited number supported types
    that are universal across all platforms
    and programming languages.
    
    The value is constructible from large number of
    raw types or associated types (e.g. i32, &str, etc.).
    Getting the raw value back as string can be done
    with `to_string()` but otherwise requires a `match`.
    *
    * @return mixed
    */
    public function getKey()
    {
        return $this->key;
    }
    /**
    * Database value is a strongly types value.
    
    It is an enum of limited number supported types
    that are universal across all platforms
    and programming languages.
    
    The value is constructible from large number of
    raw types or associated types (e.g. i32, &str, etc.).
    Getting the raw value back as string can be done
    with `to_string()` but otherwise requires a `match`.
    *
    * @param mixed $key
    *
    * @return self
    */
    public function setKey($key): self
    {
        $this->initialized['key'] = true;
        $this->key = $key;
        return $this;
    }
    /**
    * Database value is a strongly types value.
    
    It is an enum of limited number supported types
    that are universal across all platforms
    and programming languages.
    
    The value is constructible from large number of
    raw types or associated types (e.g. i32, &str, etc.).
    Getting the raw value back as string can be done
    with `to_string()` but otherwise requires a `match`.
    *
    * @return mixed
    */
    public function getValue()
    {
        return $this->value;
    }
    /**
    * Database value is a strongly types value.
    
    It is an enum of limited number supported types
    that are universal across all platforms
    and programming languages.
    
    The value is constructible from large number of
    raw types or associated types (e.g. i32, &str, etc.).
    Getting the raw value back as string can be done
    with `to_string()` but otherwise requires a `match`.
    *
    * @param mixed $value
    *
    * @return self
    */
    public function setValue($value): self
    {
        $this->initialized['value'] = true;
        $this->value = $value;
        return $this;
    }
}