# # QueryConditionData

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**distance** | [**\Agnesoft\AgdbApi\Model\CountComparison**](CountComparison.md) | Distance from the search origin. Takes count comparison (e.g. Equal, GreaterThan). |
**edge_count** | [**\Agnesoft\AgdbApi\Model\CountComparison**](CountComparison.md) | Tests number of edges (from+to) of the current element. Only nodes will pass. Self-referential edges are counted twice. Takes count comparison (e.g. Equal, GreaterThan). |
**edge_count_from** | [**\Agnesoft\AgdbApi\Model\CountComparison**](CountComparison.md) | Tests the number of outgoing edges (from) of the current element. Takes count comparison (e.g. Equal, GreaterThan). |
**edge_count_to** | [**\Agnesoft\AgdbApi\Model\CountComparison**](CountComparison.md) | Tests the number of incoming edges (to) of the current element. Takes count comparison (e.g. Equal, GreaterThan). |
**ids** | [**\Agnesoft\AgdbApi\Model\QueryId[]**](QueryId.md) | Tests if the current id is in the list of ids. |
**key_value** | [**\Agnesoft\AgdbApi\Model\KeyValueComparison**](KeyValueComparison.md) | Tests if the current element has a property &#x60;key&#x60; with a value that evaluates true against &#x60;comparison&#x60;. |
**keys** | [**\Agnesoft\AgdbApi\Model\DbValue[]**](DbValue.md) | Test if the current element has **all** of the keys listed. |
**where** | [**\Agnesoft\AgdbApi\Model\QueryCondition[]**](QueryCondition.md) | Nested list of conditions (equivalent to brackets). |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
