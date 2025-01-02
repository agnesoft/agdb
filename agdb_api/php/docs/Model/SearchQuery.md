# # SearchQuery

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**algorithm** | [**\Agnesoft\AgdbApi\Model\SearchQueryAlgorithm**](SearchQueryAlgorithm.md) | Search algorithm to be used. Will be bypassed for path searches that unconditionally use A*. |
**conditions** | [**\Agnesoft\AgdbApi\Model\QueryCondition[]**](QueryCondition.md) | Set of conditions every element must satisfy to be included in the result. Some conditions also influence the search path as well. |
**destination** | [**\Agnesoft\AgdbApi\Model\QueryId**](QueryId.md) | Target element of the path search (if origin is specified) or starting element of the reverse search (if origin is not specified). |
**limit** | **int** | How many elements maximum to return. |
**offset** | **int** | How many elements that would be returned should be skipped in the result. |
**order_by** | [**\Agnesoft\AgdbApi\Model\DbKeyOrder[]**](DbKeyOrder.md) | Order of the elements in the result. The sorting happens before &#x60;offset&#x60; and &#x60;limit&#x60; are applied. |
**origin** | [**\Agnesoft\AgdbApi\Model\QueryId**](QueryId.md) | Starting element of the search. |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
