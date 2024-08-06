# # SearchQuery

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**algorithm** | [**\Agdb\Model\SearchQueryAlgorithm**](SearchQueryAlgorithm.md) |  |
**conditions** | [**\Agdb\Model\QueryCondition[]**](QueryCondition.md) | Set of conditions every element must satisfy to be included in the result. Some conditions also influence the search path as well. |
**destination** | [**\Agdb\Model\QueryId**](QueryId.md) |  |
**limit** | **int** | How many elements maximum to return. |
**offset** | **int** | How many elements that would be returned should be skipped in the result. |
**order_by** | [**\Agdb\Model\DbKeyOrder[]**](DbKeyOrder.md) | Order of the elements in the result. The sorting happens before &#x60;offset&#x60; and &#x60;limit&#x60; are applied. |
**origin** | [**\Agdb\Model\QueryId**](QueryId.md) |  |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
