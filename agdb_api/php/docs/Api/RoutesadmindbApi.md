# Agdb\RoutesadmindbApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**adminDbAdd()**](RoutesadmindbApi.md#adminDbAdd) | **POST** /api/v1/admin/db/{owner}/{db}/add |  |
| [**adminDbAudit()**](RoutesadmindbApi.md#adminDbAudit) | **GET** /api/v1/admin/db/{owner}/{db}/audit |  |
| [**adminDbBackup()**](RoutesadmindbApi.md#adminDbBackup) | **POST** /api/v1/admin/db/{owner}/{db}/backup |  |
| [**adminDbCopy()**](RoutesadmindbApi.md#adminDbCopy) | **POST** /api/v1/admin/db/{owner}/{db}/copy |  |
| [**adminDbDelete()**](RoutesadmindbApi.md#adminDbDelete) | **DELETE** /api/v1/admin/db/{owner}/{db}/delete |  |
| [**adminDbExec()**](RoutesadmindbApi.md#adminDbExec) | **POST** /api/v1/admin/db/{owner}/{db}/exec |  |
| [**adminDbList()**](RoutesadmindbApi.md#adminDbList) | **GET** /api/v1/admin/db/list |  |
| [**adminDbOptimize()**](RoutesadmindbApi.md#adminDbOptimize) | **POST** /api/v1/admin/db/{owner}/{db}/optimize |  |
| [**adminDbRemove()**](RoutesadmindbApi.md#adminDbRemove) | **DELETE** /api/v1/admin/db/{owner}/{db}/remove |  |
| [**adminDbRename()**](RoutesadmindbApi.md#adminDbRename) | **POST** /api/v1/admin/db/{owner}/{db}/rename |  |
| [**adminDbRestore()**](RoutesadmindbApi.md#adminDbRestore) | **POST** /api/v1/db/admin/{owner}/{db}/restore |  |


## `adminDbAdd()`

```php
adminDbAdd($owner, $db, $db_type)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$db_type = new \Agdb\Model\DbType(); // DbType

try {
    $apiInstance->adminDbAdd($owner, $db, $db_type);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbAdd: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |
| **db_type** | [**DbType**](../Model/.md)|  | |

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

## `adminDbAudit()`

```php
adminDbAudit($owner, $db): \Agdb\Model\QueryAudit[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $result = $apiInstance->adminDbAudit($owner, $db);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbAudit: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |

### Return type

[**\Agdb\Model\QueryAudit[]**](../Model/QueryAudit.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminDbBackup()`

```php
adminDbBackup($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->adminDbBackup($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbBackup: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
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

## `adminDbCopy()`

```php
adminDbCopy($owner, $db, $new_name)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$new_name = 'new_name_example'; // string

try {
    $apiInstance->adminDbCopy($owner, $db, $new_name);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbCopy: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **new_name** | **string**|  | |

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

## `adminDbDelete()`

```php
adminDbDelete($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->adminDbDelete($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbDelete: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
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

## `adminDbExec()`

```php
adminDbExec($owner, $db, $query_type): \Agdb\Model\QueryResult[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$query_type = array(new \Agdb\Model\QueryType()); // \Agdb\Model\QueryType[]

try {
    $result = $apiInstance->adminDbExec($owner, $db, $query_type);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbExec: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **query_type** | [**\Agdb\Model\QueryType[]**](../Model/QueryType.md)|  | |

### Return type

[**\Agdb\Model\QueryResult[]**](../Model/QueryResult.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminDbList()`

```php
adminDbList(): \Agdb\Model\ServerDatabase[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $result = $apiInstance->adminDbList();
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**\Agdb\Model\ServerDatabase[]**](../Model/ServerDatabase.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminDbOptimize()`

```php
adminDbOptimize($owner, $db): \Agdb\Model\ServerDatabase
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $result = $apiInstance->adminDbOptimize($owner, $db);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbOptimize: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |

### Return type

[**\Agdb\Model\ServerDatabase**](../Model/ServerDatabase.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminDbRemove()`

```php
adminDbRemove($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->adminDbRemove($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbRemove: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
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

## `adminDbRename()`

```php
adminDbRename($owner, $db, $new_name)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$new_name = 'new_name_example'; // string

try {
    $apiInstance->adminDbRename($owner, $db, $new_name);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbRename: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **new_name** | **string**|  | |

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

## `adminDbRestore()`

```php
adminDbRestore($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadmindbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->adminDbRestore($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadmindbApi->adminDbRestore: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
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
