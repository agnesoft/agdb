# Agdb\RoutesdbApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**dbAdd()**](RoutesdbApi.md#dbAdd) | **POST** /api/v1/db/{owner}/{db}/add |  |
| [**dbAudit()**](RoutesdbApi.md#dbAudit) | **GET** /api/v1/db/{owner}/{db}/audit |  |
| [**dbBackup()**](RoutesdbApi.md#dbBackup) | **POST** /api/v1/db/{owner}/{db}/backup |  |
| [**dbClear()**](RoutesdbApi.md#dbClear) | **POST** /api/v1/db/{owner}/{db}/clear |  |
| [**dbCopy()**](RoutesdbApi.md#dbCopy) | **POST** /api/v1/db/{owner}/{db}/copy |  |
| [**dbDelete()**](RoutesdbApi.md#dbDelete) | **DELETE** /api/v1/db/{owner}/{db}/delete |  |
| [**dbExec()**](RoutesdbApi.md#dbExec) | **POST** /api/v1/db/{owner}/{db}/exec |  |
| [**dbList()**](RoutesdbApi.md#dbList) | **GET** /api/v1/db/list |  |
| [**dbOptimize()**](RoutesdbApi.md#dbOptimize) | **POST** /api/v1/db/{owner}/{db}/optimize |  |
| [**dbRemove()**](RoutesdbApi.md#dbRemove) | **DELETE** /api/v1/db/{owner}/{db}/remove |  |
| [**dbRename()**](RoutesdbApi.md#dbRename) | **POST** /api/v1/db/{owner}/{db}/rename |  |
| [**dbRestore()**](RoutesdbApi.md#dbRestore) | **POST** /api/v1/db/{owner}/{db}/restore |  |


## `dbAdd()`

```php
dbAdd($owner, $db, $db_type)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$db_type = new \Agdb\Model\DbType(); // DbType

try {
    $apiInstance->dbAdd($owner, $db, $db_type);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbAdd: ', $e->getMessage(), PHP_EOL;
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

## `dbAudit()`

```php
dbAudit($owner, $db): \Agdb\Model\QueryAudit[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $result = $apiInstance->dbAudit($owner, $db);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbAudit: ', $e->getMessage(), PHP_EOL;
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

## `dbBackup()`

```php
dbBackup($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->dbBackup($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbBackup: ', $e->getMessage(), PHP_EOL;
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

## `dbClear()`

```php
dbClear($owner, $db, $resource): \Agdb\Model\ServerDatabase
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$resource = new \Agdb\Model\DbResource(); // DbResource

try {
    $result = $apiInstance->dbClear($owner, $db, $resource);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbClear: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |
| **resource** | [**DbResource**](../Model/.md)|  | |

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

## `dbCopy()`

```php
dbCopy($owner, $db, $new_name)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$new_name = 'new_name_example'; // string

try {
    $apiInstance->dbCopy($owner, $db, $new_name);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbCopy: ', $e->getMessage(), PHP_EOL;
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

## `dbDelete()`

```php
dbDelete($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->dbDelete($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbDelete: ', $e->getMessage(), PHP_EOL;
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

## `dbExec()`

```php
dbExec($owner, $db, $query_type): \Agdb\Model\QueryResult[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$query_type = array(new \Agdb\Model\QueryType()); // \Agdb\Model\QueryType[]

try {
    $result = $apiInstance->dbExec($owner, $db, $query_type);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbExec: ', $e->getMessage(), PHP_EOL;
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

## `dbList()`

```php
dbList(): \Agdb\Model\ServerDatabase[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $result = $apiInstance->dbList();
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbList: ', $e->getMessage(), PHP_EOL;
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

## `dbOptimize()`

```php
dbOptimize($owner, $db): \Agdb\Model\ServerDatabase
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $result = $apiInstance->dbOptimize($owner, $db);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbOptimize: ', $e->getMessage(), PHP_EOL;
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

## `dbRemove()`

```php
dbRemove($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->dbRemove($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbRemove: ', $e->getMessage(), PHP_EOL;
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

## `dbRename()`

```php
dbRename($owner, $db, $new_name)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$new_name = 'new_name_example'; // string

try {
    $apiInstance->dbRename($owner, $db, $new_name);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbRename: ', $e->getMessage(), PHP_EOL;
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

## `dbRestore()`

```php
dbRestore($owner, $db)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name

try {
    $apiInstance->dbRestore($owner, $db);
} catch (Exception $e) {
    echo 'Exception when calling RoutesdbApi->dbRestore: ', $e->getMessage(), PHP_EOL;
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
