# # QueryType

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**insert_alias** | [**\Agnesoft\AgdbApi\Model\InsertAliasesQuery**](InsertAliasesQuery.md) |  |
**insert_edges** | [**\Agnesoft\AgdbApi\Model\InsertEdgesQuery**](InsertEdgesQuery.md) |  |
**insert_index** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | Query to create a new index on a given key. |
**insert_nodes** | [**\Agnesoft\AgdbApi\Model\InsertNodesQuery**](InsertNodesQuery.md) |  |
**insert_values** | [**\Agnesoft\AgdbApi\Model\InsertValuesQuery**](InsertValuesQuery.md) |  |
**remove** | [**\Agnesoft\AgdbApi\Model\QueryIds**](QueryIds.md) | Query to remove database elements (nodes &amp; edges). It is not an error if any of the &#x60;ids&#x60; do not already exist.  All properties associated with a given element are also removed.  If removing nodes all of its incoming and outgoing edges are also removed along with their properties. |
**remove_aliases** | **string[]** | Query to remove aliases from the database. It is not an error if an alias to be removed already does not exist.  The result will be a negative number signifying how many aliases have been actually removed. |
**remove_index** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | Query to create a new index on a given key. |
**remove_values** | [**\Agnesoft\AgdbApi\Model\SelectValuesQuery**](SelectValuesQuery.md) | Query to remove properties from existing elements in the database. All of the specified &#x60;ids&#x60; must exist in the database however they do not need to have all the listed keys (it is NOT an error if any or all keys do not exist on any of the elements). |
**search** | [**\Agnesoft\AgdbApi\Model\SearchQuery**](SearchQuery.md) |  |
**select_aliases** | [**\Agnesoft\AgdbApi\Model\QueryIds**](QueryIds.md) | Query to select aliases of given ids. All of the ids must exist in the database and have an alias.  The result will be number of returned aliases and list of elements with a single property &#x60;String(\&quot;alias\&quot;)&#x60; holding the value &#x60;String&#x60;. |
**select_all_aliases** | **object** | Query to select all aliases in the database.  The result will be number of returned aliases and list of elements with a single property &#x60;String(\&quot;alias\&quot;)&#x60; holding the value &#x60;String&#x60;. |
**select_edge_count** | [**\Agnesoft\AgdbApi\Model\SelectEdgeCountQuery**](SelectEdgeCountQuery.md) |  |
**select_indexes** | **object** | Query to select all indexes in the database.  The result will be number of returned indexes and single element with index 0 and the properties corresponding to the names of the indexes (keys) with &#x60;u64&#x60; values representing number of indexed values in each index. |
**select_keys** | [**\Agnesoft\AgdbApi\Model\QueryIds**](QueryIds.md) | Query to select only property keys of given ids. All of the ids must exist in the database.  The result will be number of elements returned and the list of elements with all properties except all values will be empty. |
**select_key_count** | [**\Agnesoft\AgdbApi\Model\QueryIds**](QueryIds.md) | Query to select number of properties (key count) of given ids. All of the ids must exist in the database.  The result will be number of elements returned and the list of elements with a single property &#x60;String(\&quot;key_count\&quot;)&#x60; with a value &#x60;u64&#x60;. |
**select_node_count** | **object** | Query to select number of nodes in the database.  The result will be 1 and elements with a single element of id 0 and a single property &#x60;String(\&quot;node_count\&quot;)&#x60; with a value &#x60;u64&#x60; represneting number of nodes in teh database. |
**select_values** | [**\Agnesoft\AgdbApi\Model\SelectValuesQuery**](SelectValuesQuery.md) |  |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
