# Agdb\RoutesadmindbuserApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**adminDbUserAdd()**](RoutesadmindbuserApi.md#adminDbUserAdd) | **PUT** /api/v1/admin/db/{owner}/{db}/user/{username}/add |  |
| [**adminDbUserList()**](RoutesadmindbuserApi.md#adminDbUserList) | **GET** /api/v1/admin/db/{owner}/{db}/user/list |  |
| [**adminDbUserRemove()**](RoutesadmindbuserApi.md#adminDbUserRemove) | **DELETE** /api/v1/admin/db/{owner}/{db}/user/{username}/remove |  |


## `adminDbUserAdd()`

```php
adminDbUserAdd($owner, $db, $username, $db_role)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$username = 'username_example'; // string | user name
$db_role = new \Agdb\Model\DbUserRole(); // DbUserRole

try {
    $apiInstance->adminDbUserAdd($owner, $db, $username, $db_role);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbuserApi->adminDbUserAdd: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **username** | **string**| user name | |
| **db_role** | [**DbUserRole**](../Model/.md)|  | |

### Return type

void (empty response body)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminDbUserList()`

```php
adminDbUserList($owner, $db): \Agdb\Model\DbUser[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name

try {
    $result = $apiInstance->adminDbUserList($owner, $db);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbuserApi->adminDbUserList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |

### Return type

[**\Agdb\Model\DbUser[]**](../Model/DbUser.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminDbUserRemove()`

```php
adminDbUserRemove($owner, $db, $username)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$username = 'username_example'; // string | user name

try {
    $apiInstance->adminDbUserRemove($owner, $db, $username);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbuserApi->adminDbUserRemove: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **username** | **string**| user name | |

### Return type

void (empty response body)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)
