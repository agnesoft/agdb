# AmendOneOf

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**add_bitwise** | [**\Agnesoft\AgdbApi\Model\BitwiseOp**](BitwiseOp.md) | Apply a bitwise operation in an \&quot;add\&quot; context. For integer types (&#x60;i64&#x60;, &#x60;u64&#x60;) the bitwise op is applied directly, with cross-type &#x60;i64&#x60;↔&#x60;u64&#x60; interop. For bytes the op is applied element-wise (shorter operand zero-padded). For all other types falls back to &#x60;Add&#x60; semantics (concatenate, extend, etc.). If the key does not exist, falls back to a regular insert. |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
