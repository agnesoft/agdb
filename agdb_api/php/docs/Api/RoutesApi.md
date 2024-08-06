# Agdb\RoutesApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**status()**](RoutesApi.md#status) | **GET** /api/v1/status |  |


## `status()`

```php
status($cluster): \Agdb\Model\ClusterStatus[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');



$apiInstance = new Agdb\Api\RoutesApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client()
);
$cluster = True; // bool | get cluster status

try {
    $result = $apiInstance->status($cluster);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesApi->status: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **cluster** | **bool**| get cluster status | |

### Return type

[**\Agdb\Model\ClusterStatus[]**](../Model/ClusterStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)
