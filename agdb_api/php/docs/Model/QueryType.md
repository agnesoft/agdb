# # QueryType

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**insert_alias** | [**\Agnesoft\AgdbApi\Model\InsertAliasesQuery**](InsertAliasesQuery.md) |  |
**insert_edges** | [**\Agnesoft\AgdbApi\Model\InsertEdgesQuery**](InsertEdgesQuery.md) |  |
**insert_index** | [**\Agnesoft\AgdbApi\Model\InsertIndexQuery**](InsertIndexQuery.md) |  |
**insert_nodes** | [**\Agnesoft\AgdbApi\Model\InsertNodesQuery**](InsertNodesQuery.md) |  |
**insert_values** | [**\Agnesoft\AgdbApi\Model\InsertValuesQuery**](InsertValuesQuery.md) |  |
**remove** | [**\Agnesoft\AgdbApi\Model\RemoveQuery**](RemoveQuery.md) |  |
**remove_aliases** | **string[]** | Query to remove aliases from the database. It is not an error if an alias to be removed already does not exist.  The result will be a negative number signifying how many aliases have been actually removed. |
**remove_index** | [**\Agnesoft\AgdbApi\Model\RemoveIndexQuery**](RemoveIndexQuery.md) |  |
**remove_values** | [**\Agnesoft\AgdbApi\Model\RemoveValuesQuery**](RemoveValuesQuery.md) |  |
**search** | [**\Agnesoft\AgdbApi\Model\SearchQuery**](SearchQuery.md) |  |
**select_aliases** | [**\Agnesoft\AgdbApi\Model\SelectAliasesQuery**](SelectAliasesQuery.md) |  |
**select_all_aliases** | **object** | Query to select all aliases in the database.  The result will be number of returned aliases and list of elements with a single property &#x60;String(\&quot;alias\&quot;)&#x60; holding the value &#x60;String&#x60;. |
**select_edge_count** | [**\Agnesoft\AgdbApi\Model\SelectEdgeCountQuery**](SelectEdgeCountQuery.md) |  |
**select_indexes** | **object** | Query to select all indexes in the database.  The result will be number of returned indexes and single element with index 0 and the properties corresponding to the names of the indexes (keys) with &#x60;u64&#x60; values representing number of indexed values in each index. |
**select_keys** | [**\Agnesoft\AgdbApi\Model\SelectKeysQuery**](SelectKeysQuery.md) |  |
**select_key_count** | [**\Agnesoft\AgdbApi\Model\SelectKeyCountQuery**](SelectKeyCountQuery.md) |  |
**select_node_count** | **object** | Query to select number of nodes in the database.  The result will be 1 and elements with a single element of id 0 and a single property &#x60;String(\&quot;node_count\&quot;)&#x60; with a value &#x60;u64&#x60; represneting number of nodes in teh database. |
**select_values** | [**\Agnesoft\AgdbApi\Model\SelectValuesQuery**](SelectValuesQuery.md) |  |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
