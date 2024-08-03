# Agdb\RoutesdbuserApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**dbUserAdd()**](RoutesdbuserApi.md#dbUserAdd) | **POST** /api/v1/db/{owner}/{db}/user/{username}/add |  |
| [**dbUserList()**](RoutesdbuserApi.md#dbUserList) | **GET** /api/v1/db/{owner}/{db}/user/list |  |
| [**dbUserRemove()**](RoutesdbuserApi.md#dbUserRemove) | **POST** /api/v1/db/{owner}/{db}/user/{username}/remove |  |


## `dbUserAdd()`

```php
dbUserAdd($owner, $db, $username, $db_role)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbuserApi(
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
    $apiInstance->dbUserAdd($owner, $db, $username, $db_role);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbuserApi->dbUserAdd: ', $e->getMessage(), PHP_EOL;
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

## `dbUserList()`

```php
dbUserList($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->dbUserList($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbuserApi->dbUserList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |

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

## `dbUserRemove()`

```php
dbUserRemove($owner, $db, $username)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$username = 'username_example'; // string | user name

try {
    $apiInstance->dbUserRemove($owner, $db, $username);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbuserApi->dbUserRemove: ', $e->getMessage(), PHP_EOL;
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
