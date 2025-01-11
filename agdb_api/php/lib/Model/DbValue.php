<?php
/**
 * DbValue
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
 * The version of the OpenAPI document: 0.10.0
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.10.0
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
 * DbValue Class Doc Comment
 *
 * @category Class
 * @description Database value is a strongly types value.  It is an enum of limited number supported types that are universal across all platforms and programming languages.  The value is constructible from large number of raw types or associated types (e.g. i32, &amp;str, etc.). Getting the raw value back as string can be done with &#x60;to_string()&#x60; but otherwise requires a &#x60;match&#x60;.
 * @package  Agnesoft\AgdbApi
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 * @implements \ArrayAccess<string, mixed>
 */
class DbValue implements ModelInterface, ArrayAccess, \JsonSerializable
{
    public const DISCRIMINATOR = null;

    /**
      * The original name of the model.
      *
      * @var string
      */
    protected static $openAPIModelName = 'DbValue';

    /**
      * Array of property to type mappings. Used for (de)serialization
      *
      * @var string[]
      */
    protected static $openAPITypes = [
        'bytes' => 'int[]',
        'i64' => 'int',
        'u64' => 'int',
        'f64' => 'float',
        'string' => 'string',
        'vec_i64' => 'int[]',
        'vec_u64' => 'int[]',
        'vec_f64' => 'float[]',
        'vec_string' => 'string[]'
    ];

    /**
      * Array of property to format mappings. Used for (de)serialization
      *
      * @var string[]
      * @phpstan-var array<string, string|null>
      * @psalm-var array<string, string|null>
      */
    protected static $openAPIFormats = [
        'bytes' => 'int32',
        'i64' => 'int64',
        'u64' => 'int64',
        'f64' => 'double',
        'string' => null,
        'vec_i64' => 'int64',
        'vec_u64' => 'int64',
        'vec_f64' => 'double',
        'vec_string' => null
    ];

    /**
      * Array of nullable properties. Used for (de)serialization
      *
      * @var boolean[]
      */
    protected static array $openAPINullables = [
        'bytes' => false,
        'i64' => false,
        'u64' => false,
        'f64' => false,
        'string' => false,
        'vec_i64' => false,
        'vec_u64' => false,
        'vec_f64' => false,
        'vec_string' => false
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
        'bytes' => 'Bytes',
        'i64' => 'I64',
        'u64' => 'U64',
        'f64' => 'F64',
        'string' => 'String',
        'vec_i64' => 'VecI64',
        'vec_u64' => 'VecU64',
        'vec_f64' => 'VecF64',
        'vec_string' => 'VecString'
    ];

    /**
     * Array of attributes to setter functions (for deserialization of responses)
     *
     * @var string[]
     */
    protected static $setters = [
        'bytes' => 'setBytes',
        'i64' => 'setI64',
        'u64' => 'setU64',
        'f64' => 'setF64',
        'string' => 'setString',
        'vec_i64' => 'setVecI64',
        'vec_u64' => 'setVecU64',
        'vec_f64' => 'setVecF64',
        'vec_string' => 'setVecString'
    ];

    /**
     * Array of attributes to getter functions (for serialization of requests)
     *
     * @var string[]
     */
    protected static $getters = [
        'bytes' => 'getBytes',
        'i64' => 'getI64',
        'u64' => 'getU64',
        'f64' => 'getF64',
        'string' => 'getString',
        'vec_i64' => 'getVecI64',
        'vec_u64' => 'getVecU64',
        'vec_f64' => 'getVecF64',
        'vec_string' => 'getVecString'
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
        $this->setIfExists('bytes', $data ?? [], null);
        $this->setIfExists('i64', $data ?? [], null);
        $this->setIfExists('u64', $data ?? [], null);
        $this->setIfExists('f64', $data ?? [], null);
        $this->setIfExists('string', $data ?? [], null);
        $this->setIfExists('vec_i64', $data ?? [], null);
        $this->setIfExists('vec_u64', $data ?? [], null);
        $this->setIfExists('vec_f64', $data ?? [], null);
        $this->setIfExists('vec_string', $data ?? [], null);
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

        if ($this->container['bytes'] === null) {
            $invalidProperties[] = "'bytes' can't be null";
        }
        if ($this->container['i64'] === null) {
            $invalidProperties[] = "'i64' can't be null";
        }
        if ($this->container['u64'] === null) {
            $invalidProperties[] = "'u64' can't be null";
        }
        if (($this->container['u64'] < 0)) {
            $invalidProperties[] = "invalid value for 'u64', must be bigger than or equal to 0.";
        }

        if ($this->container['f64'] === null) {
            $invalidProperties[] = "'f64' can't be null";
        }
        if ($this->container['string'] === null) {
            $invalidProperties[] = "'string' can't be null";
        }
        if ($this->container['vec_i64'] === null) {
            $invalidProperties[] = "'vec_i64' can't be null";
        }
        if ($this->container['vec_u64'] === null) {
            $invalidProperties[] = "'vec_u64' can't be null";
        }
        if ($this->container['vec_f64'] === null) {
            $invalidProperties[] = "'vec_f64' can't be null";
        }
        if ($this->container['vec_string'] === null) {
            $invalidProperties[] = "'vec_string' can't be null";
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
     * Gets bytes
     *
     * @return int[]
     */
    public function getBytes()
    {
        return $this->container['bytes'];
    }

    /**
     * Sets bytes
     *
     * @param int[] $bytes Byte array, sometimes referred to as blob
     *
     * @return self
     */
    public function setBytes($bytes)
    {
        if (is_null($bytes)) {
            throw new \InvalidArgumentException('non-nullable bytes cannot be null');
        }
        $this->container['bytes'] = $bytes;

        return $this;
    }

    /**
     * Gets i64
     *
     * @return int
     */
    public function getI64()
    {
        return $this->container['i64'];
    }

    /**
     * Sets i64
     *
     * @param int $i64 64-bit wide signed integer
     *
     * @return self
     */
    public function setI64($i64)
    {
        if (is_null($i64)) {
            throw new \InvalidArgumentException('non-nullable i64 cannot be null');
        }
        $this->container['i64'] = $i64;

        return $this;
    }

    /**
     * Gets u64
     *
     * @return int
     */
    public function getU64()
    {
        return $this->container['u64'];
    }

    /**
     * Sets u64
     *
     * @param int $u64 64-bit wide unsigned integer
     *
     * @return self
     */
    public function setU64($u64)
    {
        if (is_null($u64)) {
            throw new \InvalidArgumentException('non-nullable u64 cannot be null');
        }

        if (($u64 < 0)) {
            throw new \InvalidArgumentException('invalid value for $u64 when calling DbValue., must be bigger than or equal to 0.');
        }

        $this->container['u64'] = $u64;

        return $this;
    }

    /**
     * Gets f64
     *
     * @return float
     */
    public function getF64()
    {
        return $this->container['f64'];
    }

    /**
     * Sets f64
     *
     * @param float $f64 Database float is a wrapper around `f64` to provide functionality like comparison. The comparison is using `total_cmp` standard library function. See its [docs](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp) to understand how it handles NaNs and other edge cases of floating point numbers.
     *
     * @return self
     */
    public function setF64($f64)
    {
        if (is_null($f64)) {
            throw new \InvalidArgumentException('non-nullable f64 cannot be null');
        }
        $this->container['f64'] = $f64;

        return $this;
    }

    /**
     * Gets string
     *
     * @return string
     */
    public function getString()
    {
        return $this->container['string'];
    }

    /**
     * Sets string
     *
     * @param string $string UTF-8 string
     *
     * @return self
     */
    public function setString($string)
    {
        if (is_null($string)) {
            throw new \InvalidArgumentException('non-nullable string cannot be null');
        }
        $this->container['string'] = $string;

        return $this;
    }

    /**
     * Gets vec_i64
     *
     * @return int[]
     */
    public function getVecI64()
    {
        return $this->container['vec_i64'];
    }

    /**
     * Sets vec_i64
     *
     * @param int[] $vec_i64 List of 64-bit wide signed integers
     *
     * @return self
     */
    public function setVecI64($vec_i64)
    {
        if (is_null($vec_i64)) {
            throw new \InvalidArgumentException('non-nullable vec_i64 cannot be null');
        }
        $this->container['vec_i64'] = $vec_i64;

        return $this;
    }

    /**
     * Gets vec_u64
     *
     * @return int[]
     */
    public function getVecU64()
    {
        return $this->container['vec_u64'];
    }

    /**
     * Sets vec_u64
     *
     * @param int[] $vec_u64 List of 64-bit wide unsigned integers
     *
     * @return self
     */
    public function setVecU64($vec_u64)
    {
        if (is_null($vec_u64)) {
            throw new \InvalidArgumentException('non-nullable vec_u64 cannot be null');
        }
        $this->container['vec_u64'] = $vec_u64;

        return $this;
    }

    /**
     * Gets vec_f64
     *
     * @return float[]
     */
    public function getVecF64()
    {
        return $this->container['vec_f64'];
    }

    /**
     * Sets vec_f64
     *
     * @param float[] $vec_f64 List of 64-bit floating point numbers
     *
     * @return self
     */
    public function setVecF64($vec_f64)
    {
        if (is_null($vec_f64)) {
            throw new \InvalidArgumentException('non-nullable vec_f64 cannot be null');
        }
        $this->container['vec_f64'] = $vec_f64;

        return $this;
    }

    /**
     * Gets vec_string
     *
     * @return string[]
     */
    public function getVecString()
    {
        return $this->container['vec_string'];
    }

    /**
     * Sets vec_string
     *
     * @param string[] $vec_string List of UTF-8 strings
     *
     * @return self
     */
    public function setVecString($vec_string)
    {
        if (is_null($vec_string)) {
            throw new \InvalidArgumentException('non-nullable vec_string cannot be null');
        }
        $this->container['vec_string'] = $vec_string;

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


