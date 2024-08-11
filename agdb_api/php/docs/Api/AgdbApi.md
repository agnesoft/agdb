# Agnesoft\\AgdbApi\AgdbApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**adminDbAdd()**](AgdbApi.md#adminDbAdd) | **POST** /api/v1/admin/db/{owner}/{db}/add |  |
| [**adminDbAudit()**](AgdbApi.md#adminDbAudit) | **GET** /api/v1/admin/db/{owner}/{db}/audit |  |
| [**adminDbBackup()**](AgdbApi.md#adminDbBackup) | **POST** /api/v1/admin/db/{owner}/{db}/backup |  |
| [**adminDbCopy()**](AgdbApi.md#adminDbCopy) | **POST** /api/v1/admin/db/{owner}/{db}/copy |  |
| [**adminDbDelete()**](AgdbApi.md#adminDbDelete) | **DELETE** /api/v1/admin/db/{owner}/{db}/delete |  |
| [**adminDbExec()**](AgdbApi.md#adminDbExec) | **POST** /api/v1/admin/db/{owner}/{db}/exec |  |
| [**adminDbList()**](AgdbApi.md#adminDbList) | **GET** /api/v1/admin/db/list |  |
| [**adminDbOptimize()**](AgdbApi.md#adminDbOptimize) | **POST** /api/v1/admin/db/{owner}/{db}/optimize |  |
| [**adminDbRemove()**](AgdbApi.md#adminDbRemove) | **DELETE** /api/v1/admin/db/{owner}/{db}/remove |  |
| [**adminDbRename()**](AgdbApi.md#adminDbRename) | **POST** /api/v1/admin/db/{owner}/{db}/rename |  |
| [**adminDbRestore()**](AgdbApi.md#adminDbRestore) | **POST** /api/v1/db/admin/{owner}/{db}/restore |  |
| [**adminDbUserAdd()**](AgdbApi.md#adminDbUserAdd) | **PUT** /api/v1/admin/db/{owner}/{db}/user/{username}/add |  |
| [**adminDbUserList()**](AgdbApi.md#adminDbUserList) | **GET** /api/v1/admin/db/{owner}/{db}/user/list |  |
| [**adminDbUserRemove()**](AgdbApi.md#adminDbUserRemove) | **DELETE** /api/v1/admin/db/{owner}/{db}/user/{username}/remove |  |
| [**adminShutdown()**](AgdbApi.md#adminShutdown) | **POST** /api/v1/admin/shutdown |  |
| [**adminUserAdd()**](AgdbApi.md#adminUserAdd) | **POST** /api/v1/admin/user/{username}/add |  |
| [**adminUserChangePassword()**](AgdbApi.md#adminUserChangePassword) | **PUT** /api/v1/admin/user/{username}/change_password |  |
| [**adminUserList()**](AgdbApi.md#adminUserList) | **GET** /api/v1/admin/user/list |  |
| [**adminUserRemove()**](AgdbApi.md#adminUserRemove) | **DELETE** /api/v1/admin/user/{username}/remove |  |
| [**dbAdd()**](AgdbApi.md#dbAdd) | **POST** /api/v1/db/{owner}/{db}/add |  |
| [**dbAudit()**](AgdbApi.md#dbAudit) | **GET** /api/v1/db/{owner}/{db}/audit |  |
| [**dbBackup()**](AgdbApi.md#dbBackup) | **POST** /api/v1/db/{owner}/{db}/backup |  |
| [**dbClear()**](AgdbApi.md#dbClear) | **POST** /api/v1/db/{owner}/{db}/clear |  |
| [**dbCopy()**](AgdbApi.md#dbCopy) | **POST** /api/v1/db/{owner}/{db}/copy |  |
| [**dbDelete()**](AgdbApi.md#dbDelete) | **DELETE** /api/v1/db/{owner}/{db}/delete |  |
| [**dbExec()**](AgdbApi.md#dbExec) | **POST** /api/v1/db/{owner}/{db}/exec |  |
| [**dbList()**](AgdbApi.md#dbList) | **GET** /api/v1/db/list |  |
| [**dbOptimize()**](AgdbApi.md#dbOptimize) | **POST** /api/v1/db/{owner}/{db}/optimize |  |
| [**dbRemove()**](AgdbApi.md#dbRemove) | **DELETE** /api/v1/db/{owner}/{db}/remove |  |
| [**dbRename()**](AgdbApi.md#dbRename) | **POST** /api/v1/db/{owner}/{db}/rename |  |
| [**dbRestore()**](AgdbApi.md#dbRestore) | **POST** /api/v1/db/{owner}/{db}/restore |  |
| [**dbUserAdd()**](AgdbApi.md#dbUserAdd) | **PUT** /api/v1/db/{owner}/{db}/user/{username}/add |  |
| [**dbUserList()**](AgdbApi.md#dbUserList) | **GET** /api/v1/db/{owner}/{db}/user/list |  |
| [**dbUserRemove()**](AgdbApi.md#dbUserRemove) | **POST** /api/v1/db/{owner}/{db}/user/{username}/remove |  |
| [**status()**](AgdbApi.md#status) | **GET** /api/v1/status |  |
| [**userChangePassword()**](AgdbApi.md#userChangePassword) | **PUT** /api/v1/user/change_password |  |
| [**userLogin()**](AgdbApi.md#userLogin) | **POST** /api/v1/user/login |  |
| [**userLogout()**](AgdbApi.md#userLogout) | **POST** /api/v1/user/logout |  |


## `adminDbAdd()`

```php
adminDbAdd($owner, $db, $db_type)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$db_type = new \Agnesoft\\AgdbApi\Model\DbType(); // DbType

try {
    $apiInstance->adminDbAdd($owner, $db, $db_type);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminDbAdd: ', $e->getMessage(), PHP_EOL;
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
adminDbAudit($owner, $db): \Agnesoft\\AgdbApi\Model\QueryAudit[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbAudit: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |

### Return type

[**\Agnesoft\\AgdbApi\Model\QueryAudit[]**](../Model/QueryAudit.md)

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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbBackup: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbCopy: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbDelete: ', $e->getMessage(), PHP_EOL;
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
adminDbExec($owner, $db, $query_type): \Agnesoft\\AgdbApi\Model\QueryResult[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$query_type = array(new \Agnesoft\\AgdbApi\Model\QueryType()); // \Agnesoft\\AgdbApi\Model\QueryType[]

try {
    $result = $apiInstance->adminDbExec($owner, $db, $query_type);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminDbExec: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **query_type** | [**\Agnesoft\\AgdbApi\Model\QueryType[]**](../Model/QueryType.md)|  | |

### Return type

[**\Agnesoft\\AgdbApi\Model\QueryResult[]**](../Model/QueryResult.md)

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
adminDbList(): \Agnesoft\\AgdbApi\Model\ServerDatabase[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $result = $apiInstance->adminDbList();
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminDbList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**\Agnesoft\\AgdbApi\Model\ServerDatabase[]**](../Model/ServerDatabase.md)

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
adminDbOptimize($owner, $db): \Agnesoft\\AgdbApi\Model\ServerDatabase
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbOptimize: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |

### Return type

[**\Agnesoft\\AgdbApi\Model\ServerDatabase**](../Model/ServerDatabase.md)

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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbRemove: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbRename: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbRestore: ', $e->getMessage(), PHP_EOL;
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

## `adminDbUserAdd()`

```php
adminDbUserAdd($owner, $db, $username, $db_role)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$username = 'username_example'; // string | user name
$db_role = new \Agnesoft\\AgdbApi\Model\DbUserRole(); // DbUserRole

try {
    $apiInstance->adminDbUserAdd($owner, $db, $username, $db_role);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminDbUserAdd: ', $e->getMessage(), PHP_EOL;
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
adminDbUserList($owner, $db): \Agnesoft\\AgdbApi\Model\DbUser[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbUserList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |

### Return type

[**\Agnesoft\\AgdbApi\Model\DbUser[]**](../Model/DbUser.md)

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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->adminDbUserRemove: ', $e->getMessage(), PHP_EOL;
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

## `adminShutdown()`

```php
adminShutdown()
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $apiInstance->adminShutdown();
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminShutdown: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

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

## `adminUserAdd()`

```php
adminUserAdd($username, $user_credentials)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$username = 'username_example'; // string | desired user name
$user_credentials = new \Agnesoft\\AgdbApi\Model\UserCredentials(); // \Agnesoft\\AgdbApi\Model\UserCredentials

try {
    $apiInstance->adminUserAdd($username, $user_credentials);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminUserAdd: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **username** | **string**| desired user name | |
| **user_credentials** | [**\Agnesoft\\AgdbApi\Model\UserCredentials**](../Model/UserCredentials.md)|  | |

### Return type

void (empty response body)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminUserChangePassword()`

```php
adminUserChangePassword($username, $user_credentials)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$username = 'username_example'; // string | user name
$user_credentials = new \Agnesoft\\AgdbApi\Model\UserCredentials(); // \Agnesoft\\AgdbApi\Model\UserCredentials

try {
    $apiInstance->adminUserChangePassword($username, $user_credentials);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminUserChangePassword: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **username** | **string**| user name | |
| **user_credentials** | [**\Agnesoft\\AgdbApi\Model\UserCredentials**](../Model/UserCredentials.md)|  | |

### Return type

void (empty response body)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminUserList()`

```php
adminUserList(): \Agnesoft\\AgdbApi\Model\UserStatus[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $result = $apiInstance->adminUserList();
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminUserList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**\Agnesoft\\AgdbApi\Model\UserStatus[]**](../Model/UserStatus.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `adminUserRemove()`

```php
adminUserRemove($username): \Agnesoft\\AgdbApi\Model\UserStatus[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$username = 'username_example'; // string | user name

try {
    $result = $apiInstance->adminUserRemove($username);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminUserRemove: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **username** | **string**| user name | |

### Return type

[**\Agnesoft\\AgdbApi\Model\UserStatus[]**](../Model/UserStatus.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `dbAdd()`

```php
dbAdd($owner, $db, $db_type)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$db_type = new \Agnesoft\\AgdbApi\Model\DbType(); // DbType

try {
    $apiInstance->dbAdd($owner, $db, $db_type);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->dbAdd: ', $e->getMessage(), PHP_EOL;
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
dbAudit($owner, $db): \Agnesoft\\AgdbApi\Model\QueryAudit[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbAudit: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |

### Return type

[**\Agnesoft\\AgdbApi\Model\QueryAudit[]**](../Model/QueryAudit.md)

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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbBackup: ', $e->getMessage(), PHP_EOL;
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
dbClear($owner, $db, $resource): \Agnesoft\\AgdbApi\Model\ServerDatabase
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$resource = new \Agnesoft\\AgdbApi\Model\DbResource(); // DbResource

try {
    $result = $apiInstance->dbClear($owner, $db, $resource);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->dbClear: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |
| **resource** | [**DbResource**](../Model/.md)|  | |

### Return type

[**\Agnesoft\\AgdbApi\Model\ServerDatabase**](../Model/ServerDatabase.md)

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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbCopy: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbDelete: ', $e->getMessage(), PHP_EOL;
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
dbExec($owner, $db, $query_type): \Agnesoft\\AgdbApi\Model\QueryResult[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$query_type = array(new \Agnesoft\\AgdbApi\Model\QueryType()); // \Agnesoft\\AgdbApi\Model\QueryType[]

try {
    $result = $apiInstance->dbExec($owner, $db, $query_type);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->dbExec: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| db owner user name | |
| **db** | **string**| db name | |
| **query_type** | [**\Agnesoft\\AgdbApi\Model\QueryType[]**](../Model/QueryType.md)|  | |

### Return type

[**\Agnesoft\\AgdbApi\Model\QueryResult[]**](../Model/QueryResult.md)

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
dbList(): \Agnesoft\\AgdbApi\Model\ServerDatabase[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $result = $apiInstance->dbList();
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->dbList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**\Agnesoft\\AgdbApi\Model\ServerDatabase[]**](../Model/ServerDatabase.md)

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
dbOptimize($owner, $db): \Agnesoft\\AgdbApi\Model\ServerDatabase
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbOptimize: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **owner** | **string**| user name | |
| **db** | **string**| db name | |

### Return type

[**\Agnesoft\\AgdbApi\Model\ServerDatabase**](../Model/ServerDatabase.md)

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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbRemove: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbRename: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbRestore: ', $e->getMessage(), PHP_EOL;
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

## `dbUserAdd()`

```php
dbUserAdd($owner, $db, $username, $db_role)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | db owner user name
$db = 'db_example'; // string | db name
$username = 'username_example'; // string | user name
$db_role = new \Agnesoft\\AgdbApi\Model\DbUserRole(); // DbUserRole

try {
    $apiInstance->dbUserAdd($owner, $db, $username, $db_role);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->dbUserAdd: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbUserList: ', $e->getMessage(), PHP_EOL;
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
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
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
    echo 'Exception when calling AgdbApi->dbUserRemove: ', $e->getMessage(), PHP_EOL;
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

## `status()`

```php
status($cluster): \Agnesoft\\AgdbApi\Model\ClusterStatus[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');



$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client()
);
$cluster = True; // bool | get cluster status

try {
    $result = $apiInstance->status($cluster);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->status: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **cluster** | **bool**| get cluster status | |

### Return type

[**\Agnesoft\\AgdbApi\Model\ClusterStatus[]**](../Model/ClusterStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `userChangePassword()`

```php
userChangePassword($change_password)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$change_password = new \Agnesoft\\AgdbApi\Model\ChangePassword(); // \Agnesoft\\AgdbApi\Model\ChangePassword

try {
    $apiInstance->userChangePassword($change_password);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->userChangePassword: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **change_password** | [**\Agnesoft\\AgdbApi\Model\ChangePassword**](../Model/ChangePassword.md)|  | |

### Return type

void (empty response body)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `userLogin()`

```php
userLogin($user_login): string
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');



$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client()
);
$user_login = new \Agnesoft\\AgdbApi\Model\UserLogin(); // \Agnesoft\\AgdbApi\Model\UserLogin

try {
    $result = $apiInstance->userLogin($user_login);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->userLogin: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **user_login** | [**\Agnesoft\\AgdbApi\Model\UserLogin**](../Model/UserLogin.md)|  | |

### Return type

**string**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `text/plain`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)

## `userLogout()`

```php
userLogout()
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agnesoft\\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $apiInstance->userLogout();
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->userLogout: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

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
