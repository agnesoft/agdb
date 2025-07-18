<?php
/**
 * SearchQuery
 *
 * PHP version 8.1
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
 * The version of the OpenAPI document: 0.11.2
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.14.0
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
 * SearchQuery Class Doc Comment
 *
 * @category Class
 * @description Query to search for ids in the database following the graph.
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 * @implements \ArrayAccess<string, mixed>
 */
class SearchQuery implements ModelInterface, ArrayAccess, \JsonSerializable
{
    public const DISCRIMINATOR = null;

    /**
      * The original name of the model.
      *
      * @var string
      */
    protected static $openAPIModelName = 'SearchQuery';

    /**
      * Array of property to type mappings. Used for (de)serialization
      *
      * @var string[]
      */
    protected static $openAPITypes = [
        'algorithm' => '\Agnesoft\AgdbApi\Model\SearchQueryAlgorithm',
        'conditions' => '\Agnesoft\AgdbApi\Model\QueryCondition[]',
        'destination' => '\Agnesoft\AgdbApi\Model\QueryId',
        'limit' => 'int',
        'offset' => 'int',
        'order_by' => '\Agnesoft\AgdbApi\Model\DbKeyOrder[]',
        'origin' => '\Agnesoft\AgdbApi\Model\QueryId'
    ];

    /**
      * Array of property to format mappings. Used for (de)serialization
      *
      * @var string[]
      * @phpstan-var array<string, string|null>
      * @psalm-var array<string, string|null>
      */
    protected static $openAPIFormats = [
        'algorithm' => null,
        'conditions' => null,
        'destination' => null,
        'limit' => 'int64',
        'offset' => 'int64',
        'order_by' => null,
        'origin' => null
    ];

    /**
      * Array of nullable properties. Used for (de)serialization
      *
      * @var boolean[]
      */
    protected static array $openAPINullables = [
        'algorithm' => false,
        'conditions' => false,
        'destination' => false,
        'limit' => false,
        'offset' => false,
        'order_by' => false,
        'origin' => false
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
        'algorithm' => 'algorithm',
        'conditions' => 'conditions',
        'destination' => 'destination',
        'limit' => 'limit',
        'offset' => 'offset',
        'order_by' => 'order_by',
        'origin' => 'origin'
    ];

    /**
     * Array of attributes to setter functions (for deserialization of responses)
     *
     * @var string[]
     */
    protected static $setters = [
        'algorithm' => 'setAlgorithm',
        'conditions' => 'setConditions',
        'destination' => 'setDestination',
        'limit' => 'setLimit',
        'offset' => 'setOffset',
        'order_by' => 'setOrderBy',
        'origin' => 'setOrigin'
    ];

    /**
     * Array of attributes to getter functions (for serialization of requests)
     *
     * @var string[]
     */
    protected static $getters = [
        'algorithm' => 'getAlgorithm',
        'conditions' => 'getConditions',
        'destination' => 'getDestination',
        'limit' => 'getLimit',
        'offset' => 'getOffset',
        'order_by' => 'getOrderBy',
        'origin' => 'getOrigin'
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
     * @param mixed[]|null $data Associated array of property values
     *                      initializing the model
     */
    public function __construct(?array $data = null)
    {
        $this->setIfExists('algorithm', $data ?? [], null);
        $this->setIfExists('conditions', $data ?? [], null);
        $this->setIfExists('destination', $data ?? [], null);
        $this->setIfExists('limit', $data ?? [], null);
        $this->setIfExists('offset', $data ?? [], null);
        $this->setIfExists('order_by', $data ?? [], null);
        $this->setIfExists('origin', $data ?? [], null);
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

        if ($this->container['algorithm'] === null) {
            $invalidProperties[] = "'algorithm' can't be null";
        }
        if ($this->container['conditions'] === null) {
            $invalidProperties[] = "'conditions' can't be null";
        }
        if ($this->container['destination'] === null) {
            $invalidProperties[] = "'destination' can't be null";
        }
        if ($this->container['limit'] === null) {
            $invalidProperties[] = "'limit' can't be null";
        }
        if (($this->container['limit'] < 0)) {
            $invalidProperties[] = "invalid value for 'limit', must be bigger than or equal to 0.";
        }

        if ($this->container['offset'] === null) {
            $invalidProperties[] = "'offset' can't be null";
        }
        if (($this->container['offset'] < 0)) {
            $invalidProperties[] = "invalid value for 'offset', must be bigger than or equal to 0.";
        }

        if ($this->container['order_by'] === null) {
            $invalidProperties[] = "'order_by' can't be null";
        }
        if ($this->container['origin'] === null) {
            $invalidProperties[] = "'origin' can't be null";
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
     * Gets algorithm
     *
     * @return \Agnesoft\AgdbApi\Model\SearchQueryAlgorithm
     */
    public function getAlgorithm()
    {
        return $this->container['algorithm'];
    }

    /**
     * Sets algorithm
     *
     * @param \Agnesoft\AgdbApi\Model\SearchQueryAlgorithm $algorithm Search algorithm to be used. Will be bypassed for path searches that unconditionally use A*.
     *
     * @return self
     */
    public function setAlgorithm($algorithm)
    {
        if (is_null($algorithm)) {
            throw new \InvalidArgumentException('non-nullable algorithm cannot be null');
        }
        $this->container['algorithm'] = $algorithm;

        return $this;
    }

    /**
     * Gets conditions
     *
     * @return \Agnesoft\AgdbApi\Model\QueryCondition[]
     */
    public function getConditions()
    {
        return $this->container['conditions'];
    }

    /**
     * Sets conditions
     *
     * @param \Agnesoft\AgdbApi\Model\QueryCondition[] $conditions Set of conditions every element must satisfy to be included in the result. Some conditions also influence the search path as well.
     *
     * @return self
     */
    public function setConditions($conditions)
    {
        if (is_null($conditions)) {
            throw new \InvalidArgumentException('non-nullable conditions cannot be null');
        }
        $this->container['conditions'] = $conditions;

        return $this;
    }

    /**
     * Gets destination
     *
     * @return \Agnesoft\AgdbApi\Model\QueryId
     */
    public function getDestination()
    {
        return $this->container['destination'];
    }

    /**
     * Sets destination
     *
     * @param \Agnesoft\AgdbApi\Model\QueryId $destination Target element of the path search (if origin is specified) or starting element of the reverse search (if origin is not specified).
     *
     * @return self
     */
    public function setDestination($destination)
    {
        if (is_null($destination)) {
            throw new \InvalidArgumentException('non-nullable destination cannot be null');
        }
        $this->container['destination'] = $destination;

        return $this;
    }

    /**
     * Gets limit
     *
     * @return int
     */
    public function getLimit()
    {
        return $this->container['limit'];
    }

    /**
     * Sets limit
     *
     * @param int $limit How many elements maximum to return.
     *
     * @return self
     */
    public function setLimit($limit)
    {
        if (is_null($limit)) {
            throw new \InvalidArgumentException('non-nullable limit cannot be null');
        }

        if (($limit < 0)) {
            throw new \InvalidArgumentException('invalid value for $limit when calling SearchQuery., must be bigger than or equal to 0.');
        }

        $this->container['limit'] = $limit;

        return $this;
    }

    /**
     * Gets offset
     *
     * @return int
     */
    public function getOffset()
    {
        return $this->container['offset'];
    }

    /**
     * Sets offset
     *
     * @param int $offset How many elements that would be returned should be skipped in the result.
     *
     * @return self
     */
    public function setOffset($offset)
    {
        if (is_null($offset)) {
            throw new \InvalidArgumentException('non-nullable offset cannot be null');
        }

        if (($offset < 0)) {
            throw new \InvalidArgumentException('invalid value for $offset when calling SearchQuery., must be bigger than or equal to 0.');
        }

        $this->container['offset'] = $offset;

        return $this;
    }

    /**
     * Gets order_by
     *
     * @return \Agnesoft\AgdbApi\Model\DbKeyOrder[]
     */
    public function getOrderBy()
    {
        return $this->container['order_by'];
    }

    /**
     * Sets order_by
     *
     * @param \Agnesoft\AgdbApi\Model\DbKeyOrder[] $order_by Order of the elements in the result. The sorting happens before `offset` and `limit` are applied.
     *
     * @return self
     */
    public function setOrderBy($order_by)
    {
        if (is_null($order_by)) {
            throw new \InvalidArgumentException('non-nullable order_by cannot be null');
        }
        $this->container['order_by'] = $order_by;

        return $this;
    }

    /**
     * Gets origin
     *
     * @return \Agnesoft\AgdbApi\Model\QueryId
     */
    public function getOrigin()
    {
        return $this->container['origin'];
    }

    /**
     * Sets origin
     *
     * @param \Agnesoft\AgdbApi\Model\QueryId $origin Starting element of the search.
     *
     * @return self
     */
    public function setOrigin($origin)
    {
        if (is_null($origin)) {
            throw new \InvalidArgumentException('non-nullable origin cannot be null');
        }
        $this->container['origin'] = $origin;

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


