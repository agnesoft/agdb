# # DbValue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bytes** | **\SplFileObject** | Byte array, sometimes referred to as blob |
**i64** | **int** | 64-bit wide signed integer |
**u64** | **int** | 64-bit wide unsigned integer |
**f64** | **float** | Database float is a wrapper around &#x60;f64&#x60; to provide functionality like comparison. The comparison is using &#x60;total_cmp&#x60; standard library function. See its [docs](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp) to understand how it handles NaNs and other edge cases of floating point numbers. |
**string** | **string** | UTF-8 string |
**vec_i64** | **int[]** | List of 64-bit wide signed integers |
**vec_u64** | **int[]** | List of 64-bit wide unsigned integers |
**vec_f64** | **float[]** | List of 64-bit floating point numbers |
**vec_string** | **string[]** | List of UTF-8 strings |

[[Back to Model list]](../../README.md#models) [[Back to API list]](../../README.md#endpoints) [[Back to README]](../../README.md)
