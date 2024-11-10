<?php
/**
 * QueryConditionData
 *
 * PHP version 7.4
 *
 * @category Class
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 */

/**
 * agdb_server
 *
 * Agnesoft Graph Database Server
 *
 * The version of the OpenAPI document: 0.9.2
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.9.0
 */

/**
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

namespace Agnesoft\AgdbApi\Model;

use \ArrayAccess;
use \Agnesoft\AgdbApi\ObjectSerializer;

/**
 * QueryConditionData Class Doc Comment
 *
 * @category Class
 * @description Query condition data
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 * @implements \ArrayAccess<string, mixed>
 */
class QueryConditionData implements ModelInterface, ArrayAccess, \JsonSerializable
{
    public const DISCRIMINATOR = null;

    /**
      * The original name of the model.
      *
      * @var string
      */
    protected static $openAPIModelName = 'QueryConditionData';

    /**
      * Array of property to type mappings. Used for (de)serialization
      *
      * @var string[]
      */
    protected static $openAPITypes = [
        'distance' => '\Agnesoft\AgdbApi\Model\CountComparison',
        'edge_count' => '\Agnesoft\AgdbApi\Model\CountComparison',
        'edge_count_from' => '\Agnesoft\AgdbApi\Model\CountComparison',
        'edge_count_to' => '\Agnesoft\AgdbApi\Model\CountComparison',
        'ids' => '\Agnesoft\AgdbApi\Model\QueryId[]',
        'key_value' => '\Agnesoft\AgdbApi\Model\QueryConditionDataOneOf5KeyValue',
        'keys' => '\Agnesoft\AgdbApi\Model\DbValue[]',
        'where' => '\Agnesoft\AgdbApi\Model\QueryCondition[]'
    ];

    /**
      * Array of property to format mappings. Used for (de)serialization
      *
      * @var string[]
      * @phpstan-var array<string, string|null>
      * @psalm-var array<string, string|null>
      */
    protected static $openAPIFormats = [
        'distance' => null,
        'edge_count' => null,
        'edge_count_from' => null,
        'edge_count_to' => null,
        'ids' => null,
        'key_value' => null,
        'keys' => null,
        'where' => null
    ];

    /**
      * Array of nullable properties. Used for (de)serialization
      *
      * @var boolean[]
      */
    protected static array $openAPINullables = [
        'distance' => false,
        'edge_count' => false,
        'edge_count_from' => false,
        'edge_count_to' => false,
        'ids' => false,
        'key_value' => false,
        'keys' => false,
        'where' => false
    ];

    /**
      * If a nullable field gets set to null, insert it here
      *
      * @var boolean[]
      */
    protected array $openAPINullablesSetToNull = [];

    /**
     * Array of property to type mappings. Used for (de)serialization
     *
     * @return array
     */
    public static function openAPITypes()
    {
        return self::$openAPITypes;
    }

    /**
     * Array of property to format mappings. Used for (de)serialization
     *
     * @return array
     */
    public static function openAPIFormats()
    {
        return self::$openAPIFormats;
    }

    /**
     * Array of nullable properties
     *
     * @return array
     */
    protected static function openAPINullables(): array
    {
        return self::$openAPINullables;
    }

    /**
     * Array of nullable field names deliberately set to null
     *
     * @return boolean[]
     */
    private function getOpenAPINullablesSetToNull(): array
    {
        return $this->openAPINullablesSetToNull;
    }

    /**
     * Setter - Array of nullable field names deliberately set to null
     *
     * @param boolean[] $openAPINullablesSetToNull
     */
    private function setOpenAPINullablesSetToNull(array $openAPINullablesSetToNull): void
    {
        $this->openAPINullablesSetToNull = $openAPINullablesSetToNull;
    }

    /**
     * Checks if a property is nullable
     *
     * @param string $property
     * @return bool
     */
    public static function isNullable(string $property): bool
    {
        return self::openAPINullables()[$property] ?? false;
    }

    /**
     * Checks if a nullable property is set to null.
     *
     * @param string $property
     * @return bool
     */
    public function isNullableSetToNull(string $property): bool
    {
        return in_array($property, $this->getOpenAPINullablesSetToNull(), true);
    }

    /**
     * Array of attributes where the key is the local name,
     * and the value is the original name
     *
     * @var string[]
     */
    protected static $attributeMap = [
        'distance' => 'Distance',
        'edge_count' => 'EdgeCount',
        'edge_count_from' => 'EdgeCountFrom',
        'edge_count_to' => 'EdgeCountTo',
        'ids' => 'Ids',
        'key_value' => 'KeyValue',
        'keys' => 'Keys',
        'where' => 'Where'
    ];

    /**
     * Array of attributes to setter functions (for deserialization of responses)
     *
     * @var string[]
     */
    protected static $setters = [
        'distance' => 'setDistance',
        'edge_count' => 'setEdgeCount',
        'edge_count_from' => 'setEdgeCountFrom',
        'edge_count_to' => 'setEdgeCountTo',
        'ids' => 'setIds',
        'key_value' => 'setKeyValue',
        'keys' => 'setKeys',
        'where' => 'setWhere'
    ];

    /**
     * Array of attributes to getter functions (for serialization of requests)
     *
     * @var string[]
     */
    protected static $getters = [
        'distance' => 'getDistance',
        'edge_count' => 'getEdgeCount',
        'edge_count_from' => 'getEdgeCountFrom',
        'edge_count_to' => 'getEdgeCountTo',
        'ids' => 'getIds',
        'key_value' => 'getKeyValue',
        'keys' => 'getKeys',
        'where' => 'getWhere'
    ];

    /**
     * Array of attributes where the key is the local name,
     * and the value is the original name
     *
     * @return array
     */
    public static function attributeMap()
    {
        return self::$attributeMap;
    }

    /**
     * Array of attributes to setter functions (for deserialization of responses)
     *
     * @return array
     */
    public static function setters()
    {
        return self::$setters;
    }

    /**
     * Array of attributes to getter functions (for serialization of requests)
     *
     * @return array
     */
    public static function getters()
    {
        return self::$getters;
    }

    /**
     * The original name of the model.
     *
     * @return string
     */
    public function getModelName()
    {
        return self::$openAPIModelName;
    }


    /**
     * Associative array for storing property values
     *
     * @var mixed[]
     */
    protected $container = [];

    /**
     * Constructor
     *
     * @param mixed[] $data Associated array of property values
     *                      initializing the model
     */
    public function __construct(array $data = null)
    {
        $this->setIfExists('distance', $data ?? [], null);
        $this->setIfExists('edge_count', $data ?? [], null);
        $this->setIfExists('edge_count_from', $data ?? [], null);
        $this->setIfExists('edge_count_to', $data ?? [], null);
        $this->setIfExists('ids', $data ?? [], null);
        $this->setIfExists('key_value', $data ?? [], null);
        $this->setIfExists('keys', $data ?? [], null);
        $this->setIfExists('where', $data ?? [], null);
    }

    /**
    * Sets $this->container[$variableName] to the given data or to the given default Value; if $variableName
    * is nullable and its value is set to null in the $fields array, then mark it as "set to null" in the
    * $this->openAPINullablesSetToNull array
    *
    * @param string $variableName
    * @param array  $fields
    * @param mixed  $defaultValue
    */
    private function setIfExists(string $variableName, array $fields, $defaultValue): void
    {
        if (self::isNullable($variableName) && array_key_exists($variableName, $fields) && is_null($fields[$variableName])) {
            $this->openAPINullablesSetToNull[] = $variableName;
        }

        $this->container[$variableName] = $fields[$variableName] ?? $defaultValue;
    }

    /**
     * Show all the invalid properties with reasons.
     *
     * @return array invalid properties with reasons
     */
    public function listInvalidProperties()
    {
        $invalidProperties = [];

        if ($this->container['distance'] === null) {
            $invalidProperties[] = "'distance' can't be null";
        }
        if ($this->container['edge_count'] === null) {
            $invalidProperties[] = "'edge_count' can't be null";
        }
        if ($this->container['edge_count_from'] === null) {
            $invalidProperties[] = "'edge_count_from' can't be null";
        }
        if ($this->container['edge_count_to'] === null) {
            $invalidProperties[] = "'edge_count_to' can't be null";
        }
        if ($this->container['ids'] === null) {
            $invalidProperties[] = "'ids' can't be null";
        }
        if ($this->container['key_value'] === null) {
            $invalidProperties[] = "'key_value' can't be null";
        }
        if ($this->container['keys'] === null) {
            $invalidProperties[] = "'keys' can't be null";
        }
        if ($this->container['where'] === null) {
            $invalidProperties[] = "'where' can't be null";
        }
        return $invalidProperties;
    }

    /**
     * Validate all the properties in the model
     * return true if all passed
     *
     * @return bool True if all properties are valid
     */
    public function valid()
    {
        return count($this->listInvalidProperties()) === 0;
    }


    /**
     * Gets distance
     *
     * @return \Agnesoft\AgdbApi\Model\CountComparison
     */
    public function getDistance()
    {
        return $this->container['distance'];
    }

    /**
     * Sets distance
     *
     * @param \Agnesoft\AgdbApi\Model\CountComparison $distance distance
     *
     * @return self
     */
    public function setDistance($distance)
    {
        if (is_null($distance)) {
            throw new \InvalidArgumentException('non-nullable distance cannot be null');
        }
        $this->container['distance'] = $distance;

        return $this;
    }

    /**
     * Gets edge_count
     *
     * @return \Agnesoft\AgdbApi\Model\CountComparison
     */
    public function getEdgeCount()
    {
        return $this->container['edge_count'];
    }

    /**
     * Sets edge_count
     *
     * @param \Agnesoft\AgdbApi\Model\CountComparison $edge_count edge_count
     *
     * @return self
     */
    public function setEdgeCount($edge_count)
    {
        if (is_null($edge_count)) {
            throw new \InvalidArgumentException('non-nullable edge_count cannot be null');
        }
        $this->container['edge_count'] = $edge_count;

        return $this;
    }

    /**
     * Gets edge_count_from
     *
     * @return \Agnesoft\AgdbApi\Model\CountComparison
     */
    public function getEdgeCountFrom()
    {
        return $this->container['edge_count_from'];
    }

    /**
     * Sets edge_count_from
     *
     * @param \Agnesoft\AgdbApi\Model\CountComparison $edge_count_from edge_count_from
     *
     * @return self
     */
    public function setEdgeCountFrom($edge_count_from)
    {
        if (is_null($edge_count_from)) {
            throw new \InvalidArgumentException('non-nullable edge_count_from cannot be null');
        }
        $this->container['edge_count_from'] = $edge_count_from;

        return $this;
    }

    /**
     * Gets edge_count_to
     *
     * @return \Agnesoft\AgdbApi\Model\CountComparison
     */
    public function getEdgeCountTo()
    {
        return $this->container['edge_count_to'];
    }

    /**
     * Sets edge_count_to
     *
     * @param \Agnesoft\AgdbApi\Model\CountComparison $edge_count_to edge_count_to
     *
     * @return self
     */
    public function setEdgeCountTo($edge_count_to)
    {
        if (is_null($edge_count_to)) {
            throw new \InvalidArgumentException('non-nullable edge_count_to cannot be null');
        }
        $this->container['edge_count_to'] = $edge_count_to;

        return $this;
    }

    /**
     * Gets ids
     *
     * @return \Agnesoft\AgdbApi\Model\QueryId[]
     */
    public function getIds()
    {
        return $this->container['ids'];
    }

    /**
     * Sets ids
     *
     * @param \Agnesoft\AgdbApi\Model\QueryId[] $ids Tests if the current id is in the list of ids.
     *
     * @return self
     */
    public function setIds($ids)
    {
        if (is_null($ids)) {
            throw new \InvalidArgumentException('non-nullable ids cannot be null');
        }
        $this->container['ids'] = $ids;

        return $this;
    }

    /**
     * Gets key_value
     *
     * @return \Agnesoft\AgdbApi\Model\QueryConditionDataOneOf5KeyValue
     */
    public function getKeyValue()
    {
        return $this->container['key_value'];
    }

    /**
     * Sets key_value
     *
     * @param \Agnesoft\AgdbApi\Model\QueryConditionDataOneOf5KeyValue $key_value key_value
     *
     * @return self
     */
    public function setKeyValue($key_value)
    {
        if (is_null($key_value)) {
            throw new \InvalidArgumentException('non-nullable key_value cannot be null');
        }
        $this->container['key_value'] = $key_value;

        return $this;
    }

    /**
     * Gets keys
     *
     * @return \Agnesoft\AgdbApi\Model\DbValue[]
     */
    public function getKeys()
    {
        return $this->container['keys'];
    }

    /**
     * Sets keys
     *
     * @param \Agnesoft\AgdbApi\Model\DbValue[] $keys Test if the current element has **all** of the keys listed.
     *
     * @return self
     */
    public function setKeys($keys)
    {
        if (is_null($keys)) {
            throw new \InvalidArgumentException('non-nullable keys cannot be null');
        }
        $this->container['keys'] = $keys;

        return $this;
    }

    /**
     * Gets where
     *
     * @return \Agnesoft\AgdbApi\Model\QueryCondition[]
     */
    public function getWhere()
    {
        return $this->container['where'];
    }

    /**
     * Sets where
     *
     * @param \Agnesoft\AgdbApi\Model\QueryCondition[] $where Nested list of conditions (equivalent to brackets).
     *
     * @return self
     */
    public function setWhere($where)
    {
        if (is_null($where)) {
            throw new \InvalidArgumentException('non-nullable where cannot be null');
        }
        $this->container['where'] = $where;

        return $this;
    }
    /**
     * Returns true if offset exists. False otherwise.
     *
     * @param integer $offset Offset
     *
     * @return boolean
     */
    public function offsetExists($offset): bool
    {
        return isset($this->container[$offset]);
    }

    /**
     * Gets offset.
     *
     * @param integer $offset Offset
     *
     * @return mixed|null
     */
    #[\ReturnTypeWillChange]
    public function offsetGet($offset)
    {
        return $this->container[$offset] ?? null;
    }

    /**
     * Sets value based on offset.
     *
     * @param int|null $offset Offset
     * @param mixed    $value  Value to be set
     *
     * @return void
     */
    public function offsetSet($offset, $value): void
    {
        if (is_null($offset)) {
            $this->container[] = $value;
        } else {
            $this->container[$offset] = $value;
        }
    }

    /**
     * Unsets offset.
     *
     * @param integer $offset Offset
     *
     * @return void
     */
    public function offsetUnset($offset): void
    {
        unset($this->container[$offset]);
    }

    /**
     * Serializes the object to a value that can be serialized natively by json_encode().
     * @link https://www.php.net/manual/en/jsonserializable.jsonserialize.php
     *
     * @return mixed Returns data which can be serialized by json_encode(), which is a value
     * of any type other than a resource.
     */
    #[\ReturnTypeWillChange]
    public function jsonSerialize()
    {
       return ObjectSerializer::sanitizeForSerialization($this);
    }

    /**
     * Gets the string presentation of the object
     *
     * @return string
     */
    public function __toString()
    {
        return json_encode(
            ObjectSerializer::sanitizeForSerialization($this),
            JSON_PRETTY_PRINT
        );
    }

    /**
     * Gets a header-safe presentation of the object
     *
     * @return string
     */
    public function toHeaderValue()
    {
        return json_encode(ObjectSerializer::sanitizeForSerialization($this));
    }
}


