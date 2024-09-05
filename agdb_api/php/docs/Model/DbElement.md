# # DbElement

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**from** | **int** | Database id is a wrapper around &#x60;i64&#x60;. The id is an identifier of a database element both nodes and edges. The positive ids represent nodes, negative ids represent edges. The value of &#x60;0&#x60; is logically invalid (there cannot be element with id 0) and a default. | [optional]
**id** | **int** | Database id is a wrapper around &#x60;i64&#x60;. The id is an identifier of a database element both nodes and edges. The positive ids represent nodes, negative ids represent edges. The value of &#x60;0&#x60; is logically invalid (there cannot be element with id 0) and a default. |
**to** | **int** | Database id is a wrapper around &#x60;i64&#x60;. The id is an identifier of a database element both nodes and edges. The positive ids represent nodes, negative ids represent edges. The value of &#x60;0&#x60; is logically invalid (there cannot be element with id 0) and a default. | [optional]
**values** | [**\Agnesoft\AgdbApi\Model\DbKeyValue[]**](DbKeyValue.md) | List of key-value pairs associated with the element. |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
