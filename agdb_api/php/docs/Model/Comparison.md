# Comparison

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**equal** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property &#x3D;&#x3D; this |
**greater_than** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property &gt; this |
**greater_than_or_equal** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property &gt;&#x3D; this |
**less_than** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property &lt; this |
**less_than_or_equal** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property &lt;&#x3D; this |
**not_equal** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property !&#x3D; this |
**contains** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property.contains(this) |
**starts_with** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property.starts_with(this) |
**ends_with** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property.ends_with(this) |
**any** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property.any(this) - true if at least one element of &#x60;this&#x60; is present in the property. For scalar right-hand side, behaves identically to &#x60;Contains&#x60;. For vector right-hand side, uses existential (OR) semantics rather than universal (AND) semantics. Empty right-hand vector yields &#x60;false&#x60;. |
**regex** | [**\Agnesoft\AgdbApi\Model\DbValue**](DbValue.md) | property matches regex pattern. The inner value must be &#x60;DbValue::String&#x60; holding a valid regex pattern. Non-string values or invalid patterns yield &#x60;false&#x60;. Requires &#x60;regex&#x60; feature to be enabled; without it the comparison always yields &#x60;false&#x60;. |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
