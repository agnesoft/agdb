<?php
/**
 * QueryType
 *
 * PHP version 7.4
 *
 * @category Class
 * @package  Agdb
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 */

/**
 * agdb_server
 *
 * Agnesoft Graph Database Server
 *
 * The version of the OpenAPI document: 0.7.2
 * Generated by: https://openapi-generator.tech
 * Generator version: 7.7.0
 */

/**
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

namespace Agdb\Model;

use \ArrayAccess;
use \Agdb\ObjectSerializer;

/**
 * QueryType Class Doc Comment
 *
 * @category Class
 * @description Convenience enum for serializing/deserializing queries.
 * @package  Agdb
 * @author   OpenAPI Generator team
 * @link     https://openapi-generator.tech
 * @implements \ArrayAccess<string, mixed>
 */
class QueryType implements ModelInterface, ArrayAccess, \JsonSerializable
{
    public const DISCRIMINATOR = null;

    /**
      * The original name of the model.
      *
      * @var string
      */
    protected static $openAPIModelName = 'QueryType';

    /**
      * Array of property to type mappings. Used for (de)serialization
      *
      * @var string[]
      */
    protected static $openAPITypes = [
        'insert_alias' => '\Agdb\Model\InsertAliasesQuery',
        'insert_edges' => '\Agdb\Model\InsertEdgesQuery',
        'insert_index' => '\Agdb\Model\DbValue',
        'insert_nodes' => '\Agdb\Model\InsertNodesQuery',
        'insert_values' => '\Agdb\Model\InsertValuesQuery',
        'remove' => '\Agdb\Model\QueryIds',
        'remove_aliases' => 'string[]',
        'remove_index' => '\Agdb\Model\DbValue',
        'remove_values' => '\Agdb\Model\SelectValuesQuery',
        'search' => '\Agdb\Model\SearchQuery',
        'select_aliases' => '\Agdb\Model\QueryIds',
        'select_all_aliases' => 'object',
        'select_edge_count' => '\Agdb\Model\SelectEdgeCountQuery',
        'select_indexes' => 'object',
        'select_keys' => '\Agdb\Model\QueryIds',
        'select_key_count' => '\Agdb\Model\QueryIds',
        'select_node_count' => 'object',
        'select_values' => '\Agdb\Model\SelectValuesQuery'
    ];

    /**
      * Array of property to format mappings. Used for (de)serialization
      *
      * @var string[]
      * @phpstan-var array<string, string|null>
      * @psalm-var array<string, string|null>
      */
    protected static $openAPIFormats = [
        'insert_alias' => null,
        'insert_edges' => null,
        'insert_index' => null,
        'insert_nodes' => null,
        'insert_values' => null,
        'remove' => null,
        'remove_aliases' => null,
        'remove_index' => null,
        'remove_values' => null,
        'search' => null,
        'select_aliases' => null,
        'select_all_aliases' => null,
        'select_edge_count' => null,
        'select_indexes' => null,
        'select_keys' => null,
        'select_key_count' => null,
        'select_node_count' => null,
        'select_values' => null
    ];

    /**
      * Array of nullable properties. Used for (de)serialization
      *
      * @var boolean[]
      */
    protected static array $openAPINullables = [
        'insert_alias' => false,
        'insert_edges' => false,
        'insert_index' => false,
        'insert_nodes' => false,
        'insert_values' => false,
        'remove' => false,
        'remove_aliases' => false,
        'remove_index' => false,
        'remove_values' => false,
        'search' => false,
        'select_aliases' => false,
        'select_all_aliases' => false,
        'select_edge_count' => false,
        'select_indexes' => false,
        'select_keys' => false,
        'select_key_count' => false,
        'select_node_count' => false,
        'select_values' => false
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
        'insert_alias' => 'InsertAlias',
        'insert_edges' => 'InsertEdges',
        'insert_index' => 'InsertIndex',
        'insert_nodes' => 'InsertNodes',
        'insert_values' => 'InsertValues',
        'remove' => 'Remove',
        'remove_aliases' => 'RemoveAliases',
        'remove_index' => 'RemoveIndex',
        'remove_values' => 'RemoveValues',
        'search' => 'Search',
        'select_aliases' => 'SelectAliases',
        'select_all_aliases' => 'SelectAllAliases',
        'select_edge_count' => 'SelectEdgeCount',
        'select_indexes' => 'SelectIndexes',
        'select_keys' => 'SelectKeys',
        'select_key_count' => 'SelectKeyCount',
        'select_node_count' => 'SelectNodeCount',
        'select_values' => 'SelectValues'
    ];

    /**
     * Array of attributes to setter functions (for deserialization of responses)
     *
     * @var string[]
     */
    protected static $setters = [
        'insert_alias' => 'setInsertAlias',
        'insert_edges' => 'setInsertEdges',
        'insert_index' => 'setInsertIndex',
        'insert_nodes' => 'setInsertNodes',
        'insert_values' => 'setInsertValues',
        'remove' => 'setRemove',
        'remove_aliases' => 'setRemoveAliases',
        'remove_index' => 'setRemoveIndex',
        'remove_values' => 'setRemoveValues',
        'search' => 'setSearch',
        'select_aliases' => 'setSelectAliases',
        'select_all_aliases' => 'setSelectAllAliases',
        'select_edge_count' => 'setSelectEdgeCount',
        'select_indexes' => 'setSelectIndexes',
        'select_keys' => 'setSelectKeys',
        'select_key_count' => 'setSelectKeyCount',
        'select_node_count' => 'setSelectNodeCount',
        'select_values' => 'setSelectValues'
    ];

    /**
     * Array of attributes to getter functions (for serialization of requests)
     *
     * @var string[]
     */
    protected static $getters = [
        'insert_alias' => 'getInsertAlias',
        'insert_edges' => 'getInsertEdges',
        'insert_index' => 'getInsertIndex',
        'insert_nodes' => 'getInsertNodes',
        'insert_values' => 'getInsertValues',
        'remove' => 'getRemove',
        'remove_aliases' => 'getRemoveAliases',
        'remove_index' => 'getRemoveIndex',
        'remove_values' => 'getRemoveValues',
        'search' => 'getSearch',
        'select_aliases' => 'getSelectAliases',
        'select_all_aliases' => 'getSelectAllAliases',
        'select_edge_count' => 'getSelectEdgeCount',
        'select_indexes' => 'getSelectIndexes',
        'select_keys' => 'getSelectKeys',
        'select_key_count' => 'getSelectKeyCount',
        'select_node_count' => 'getSelectNodeCount',
        'select_values' => 'getSelectValues'
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
        $this->setIfExists('insert_alias', $data ?? [], null);
        $this->setIfExists('insert_edges', $data ?? [], null);
        $this->setIfExists('insert_index', $data ?? [], null);
        $this->setIfExists('insert_nodes', $data ?? [], null);
        $this->setIfExists('insert_values', $data ?? [], null);
        $this->setIfExists('remove', $data ?? [], null);
        $this->setIfExists('remove_aliases', $data ?? [], null);
        $this->setIfExists('remove_index', $data ?? [], null);
        $this->setIfExists('remove_values', $data ?? [], null);
        $this->setIfExists('search', $data ?? [], null);
        $this->setIfExists('select_aliases', $data ?? [], null);
        $this->setIfExists('select_all_aliases', $data ?? [], null);
        $this->setIfExists('select_edge_count', $data ?? [], null);
        $this->setIfExists('select_indexes', $data ?? [], null);
        $this->setIfExists('select_keys', $data ?? [], null);
        $this->setIfExists('select_key_count', $data ?? [], null);
        $this->setIfExists('select_node_count', $data ?? [], null);
        $this->setIfExists('select_values', $data ?? [], null);
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

        if ($this->container['insert_alias'] === null) {
            $invalidProperties[] = "'insert_alias' can't be null";
        }
        if ($this->container['insert_edges'] === null) {
            $invalidProperties[] = "'insert_edges' can't be null";
        }
        if ($this->container['insert_index'] === null) {
            $invalidProperties[] = "'insert_index' can't be null";
        }
        if ($this->container['insert_nodes'] === null) {
            $invalidProperties[] = "'insert_nodes' can't be null";
        }
        if ($this->container['insert_values'] === null) {
            $invalidProperties[] = "'insert_values' can't be null";
        }
        if ($this->container['remove'] === null) {
            $invalidProperties[] = "'remove' can't be null";
        }
        if ($this->container['remove_aliases'] === null) {
            $invalidProperties[] = "'remove_aliases' can't be null";
        }
        if ($this->container['remove_index'] === null) {
            $invalidProperties[] = "'remove_index' can't be null";
        }
        if ($this->container['remove_values'] === null) {
            $invalidProperties[] = "'remove_values' can't be null";
        }
        if ($this->container['search'] === null) {
            $invalidProperties[] = "'search' can't be null";
        }
        if ($this->container['select_aliases'] === null) {
            $invalidProperties[] = "'select_aliases' can't be null";
        }
        if ($this->container['select_all_aliases'] === null) {
            $invalidProperties[] = "'select_all_aliases' can't be null";
        }
        if ($this->container['select_edge_count'] === null) {
            $invalidProperties[] = "'select_edge_count' can't be null";
        }
        if ($this->container['select_indexes'] === null) {
            $invalidProperties[] = "'select_indexes' can't be null";
        }
        if ($this->container['select_keys'] === null) {
            $invalidProperties[] = "'select_keys' can't be null";
        }
        if ($this->container['select_key_count'] === null) {
            $invalidProperties[] = "'select_key_count' can't be null";
        }
        if ($this->container['select_node_count'] === null) {
            $invalidProperties[] = "'select_node_count' can't be null";
        }
        if ($this->container['select_values'] === null) {
            $invalidProperties[] = "'select_values' can't be null";
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
     * Gets insert_alias
     *
     * @return \Agdb\Model\InsertAliasesQuery
     */
    public function getInsertAlias()
    {
        return $this->container['insert_alias'];
    }

    /**
     * Sets insert_alias
     *
     * @param \Agdb\Model\InsertAliasesQuery $insert_alias insert_alias
     *
     * @return self
     */
    public function setInsertAlias($insert_alias)
    {
        if (is_null($insert_alias)) {
            throw new \InvalidArgumentException('non-nullable insert_alias cannot be null');
        }
        $this->container['insert_alias'] = $insert_alias;

        return $this;
    }

    /**
     * Gets insert_edges
     *
     * @return \Agdb\Model\InsertEdgesQuery
     */
    public function getInsertEdges()
    {
        return $this->container['insert_edges'];
    }

    /**
     * Sets insert_edges
     *
     * @param \Agdb\Model\InsertEdgesQuery $insert_edges insert_edges
     *
     * @return self
     */
    public function setInsertEdges($insert_edges)
    {
        if (is_null($insert_edges)) {
            throw new \InvalidArgumentException('non-nullable insert_edges cannot be null');
        }
        $this->container['insert_edges'] = $insert_edges;

        return $this;
    }

    /**
     * Gets insert_index
     *
     * @return \Agdb\Model\DbValue
     */
    public function getInsertIndex()
    {
        return $this->container['insert_index'];
    }

    /**
     * Sets insert_index
     *
     * @param \Agdb\Model\DbValue $insert_index insert_index
     *
     * @return self
     */
    public function setInsertIndex($insert_index)
    {
        if (is_null($insert_index)) {
            throw new \InvalidArgumentException('non-nullable insert_index cannot be null');
        }
        $this->container['insert_index'] = $insert_index;

        return $this;
    }

    /**
     * Gets insert_nodes
     *
     * @return \Agdb\Model\InsertNodesQuery
     */
    public function getInsertNodes()
    {
        return $this->container['insert_nodes'];
    }

    /**
     * Sets insert_nodes
     *
     * @param \Agdb\Model\InsertNodesQuery $insert_nodes insert_nodes
     *
     * @return self
     */
    public function setInsertNodes($insert_nodes)
    {
        if (is_null($insert_nodes)) {
            throw new \InvalidArgumentException('non-nullable insert_nodes cannot be null');
        }
        $this->container['insert_nodes'] = $insert_nodes;

        return $this;
    }

    /**
     * Gets insert_values
     *
     * @return \Agdb\Model\InsertValuesQuery
     */
    public function getInsertValues()
    {
        return $this->container['insert_values'];
    }

    /**
     * Sets insert_values
     *
     * @param \Agdb\Model\InsertValuesQuery $insert_values insert_values
     *
     * @return self
     */
    public function setInsertValues($insert_values)
    {
        if (is_null($insert_values)) {
            throw new \InvalidArgumentException('non-nullable insert_values cannot be null');
        }
        $this->container['insert_values'] = $insert_values;

        return $this;
    }

    /**
     * Gets remove
     *
     * @return \Agdb\Model\QueryIds
     */
    public function getRemove()
    {
        return $this->container['remove'];
    }

    /**
     * Sets remove
     *
     * @param \Agdb\Model\QueryIds $remove remove
     *
     * @return self
     */
    public function setRemove($remove)
    {
        if (is_null($remove)) {
            throw new \InvalidArgumentException('non-nullable remove cannot be null');
        }
        $this->container['remove'] = $remove;

        return $this;
    }

    /**
     * Gets remove_aliases
     *
     * @return string[]
     */
    public function getRemoveAliases()
    {
        return $this->container['remove_aliases'];
    }

    /**
     * Sets remove_aliases
     *
     * @param string[] $remove_aliases Query to remove aliases from the database. It is not an error if an alias to be removed already does not exist.  The result will be a negative number signifying how many aliases have been actually removed.
     *
     * @return self
     */
    public function setRemoveAliases($remove_aliases)
    {
        if (is_null($remove_aliases)) {
            throw new \InvalidArgumentException('non-nullable remove_aliases cannot be null');
        }
        $this->container['remove_aliases'] = $remove_aliases;

        return $this;
    }

    /**
     * Gets remove_index
     *
     * @return \Agdb\Model\DbValue
     */
    public function getRemoveIndex()
    {
        return $this->container['remove_index'];
    }

    /**
     * Sets remove_index
     *
     * @param \Agdb\Model\DbValue $remove_index remove_index
     *
     * @return self
     */
    public function setRemoveIndex($remove_index)
    {
        if (is_null($remove_index)) {
            throw new \InvalidArgumentException('non-nullable remove_index cannot be null');
        }
        $this->container['remove_index'] = $remove_index;

        return $this;
    }

    /**
     * Gets remove_values
     *
     * @return \Agdb\Model\SelectValuesQuery
     */
    public function getRemoveValues()
    {
        return $this->container['remove_values'];
    }

    /**
     * Sets remove_values
     *
     * @param \Agdb\Model\SelectValuesQuery $remove_values remove_values
     *
     * @return self
     */
    public function setRemoveValues($remove_values)
    {
        if (is_null($remove_values)) {
            throw new \InvalidArgumentException('non-nullable remove_values cannot be null');
        }
        $this->container['remove_values'] = $remove_values;

        return $this;
    }

    /**
     * Gets search
     *
     * @return \Agdb\Model\SearchQuery
     */
    public function getSearch()
    {
        return $this->container['search'];
    }

    /**
     * Sets search
     *
     * @param \Agdb\Model\SearchQuery $search search
     *
     * @return self
     */
    public function setSearch($search)
    {
        if (is_null($search)) {
            throw new \InvalidArgumentException('non-nullable search cannot be null');
        }
        $this->container['search'] = $search;

        return $this;
    }

    /**
     * Gets select_aliases
     *
     * @return \Agdb\Model\QueryIds
     */
    public function getSelectAliases()
    {
        return $this->container['select_aliases'];
    }

    /**
     * Sets select_aliases
     *
     * @param \Agdb\Model\QueryIds $select_aliases select_aliases
     *
     * @return self
     */
    public function setSelectAliases($select_aliases)
    {
        if (is_null($select_aliases)) {
            throw new \InvalidArgumentException('non-nullable select_aliases cannot be null');
        }
        $this->container['select_aliases'] = $select_aliases;

        return $this;
    }

    /**
     * Gets select_all_aliases
     *
     * @return object
     */
    public function getSelectAllAliases()
    {
        return $this->container['select_all_aliases'];
    }

    /**
     * Sets select_all_aliases
     *
     * @param object $select_all_aliases Query to select all aliases in the database.  The result will be number of returned aliases and list of elements with a single property `String(\"alias\")` holding the value `String`.
     *
     * @return self
     */
    public function setSelectAllAliases($select_all_aliases)
    {
        if (is_null($select_all_aliases)) {
            throw new \InvalidArgumentException('non-nullable select_all_aliases cannot be null');
        }
        $this->container['select_all_aliases'] = $select_all_aliases;

        return $this;
    }

    /**
     * Gets select_edge_count
     *
     * @return \Agdb\Model\SelectEdgeCountQuery
     */
    public function getSelectEdgeCount()
    {
        return $this->container['select_edge_count'];
    }

    /**
     * Sets select_edge_count
     *
     * @param \Agdb\Model\SelectEdgeCountQuery $select_edge_count select_edge_count
     *
     * @return self
     */
    public function setSelectEdgeCount($select_edge_count)
    {
        if (is_null($select_edge_count)) {
            throw new \InvalidArgumentException('non-nullable select_edge_count cannot be null');
        }
        $this->container['select_edge_count'] = $select_edge_count;

        return $this;
    }

    /**
     * Gets select_indexes
     *
     * @return object
     */
    public function getSelectIndexes()
    {
        return $this->container['select_indexes'];
    }

    /**
     * Sets select_indexes
     *
     * @param object $select_indexes Query to select all indexes in the database.  The result will be number of returned indexes and single element with index 0 and the properties corresponding to the names of the indexes (keys) with `u64` values representing number of indexed values in each index.
     *
     * @return self
     */
    public function setSelectIndexes($select_indexes)
    {
        if (is_null($select_indexes)) {
            throw new \InvalidArgumentException('non-nullable select_indexes cannot be null');
        }
        $this->container['select_indexes'] = $select_indexes;

        return $this;
    }

    /**
     * Gets select_keys
     *
     * @return \Agdb\Model\QueryIds
     */
    public function getSelectKeys()
    {
        return $this->container['select_keys'];
    }

    /**
     * Sets select_keys
     *
     * @param \Agdb\Model\QueryIds $select_keys select_keys
     *
     * @return self
     */
    public function setSelectKeys($select_keys)
    {
        if (is_null($select_keys)) {
            throw new \InvalidArgumentException('non-nullable select_keys cannot be null');
        }
        $this->container['select_keys'] = $select_keys;

        return $this;
    }

    /**
     * Gets select_key_count
     *
     * @return \Agdb\Model\QueryIds
     */
    public function getSelectKeyCount()
    {
        return $this->container['select_key_count'];
    }

    /**
     * Sets select_key_count
     *
     * @param \Agdb\Model\QueryIds $select_key_count select_key_count
     *
     * @return self
     */
    public function setSelectKeyCount($select_key_count)
    {
        if (is_null($select_key_count)) {
            throw new \InvalidArgumentException('non-nullable select_key_count cannot be null');
        }
        $this->container['select_key_count'] = $select_key_count;

        return $this;
    }

    /**
     * Gets select_node_count
     *
     * @return object
     */
    public function getSelectNodeCount()
    {
        return $this->container['select_node_count'];
    }

    /**
     * Sets select_node_count
     *
     * @param object $select_node_count Query to select number of nodes in the database.  The result will be 1 and elements with a single element of id 0 and a single property `String(\"node_count\")` with a value `u64` represneting number of nodes in teh database.
     *
     * @return self
     */
    public function setSelectNodeCount($select_node_count)
    {
        if (is_null($select_node_count)) {
            throw new \InvalidArgumentException('non-nullable select_node_count cannot be null');
        }
        $this->container['select_node_count'] = $select_node_count;

        return $this;
    }

    /**
     * Gets select_values
     *
     * @return \Agdb\Model\SelectValuesQuery
     */
    public function getSelectValues()
    {
        return $this->container['select_values'];
    }

    /**
     * Sets select_values
     *
     * @param \Agdb\Model\SelectValuesQuery $select_values select_values
     *
     * @return self
     */
    public function setSelectValues($select_values)
    {
        if (is_null($select_values)) {
            throw new \InvalidArgumentException('non-nullable select_values cannot be null');
        }
        $this->container['select_values'] = $select_values;

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

