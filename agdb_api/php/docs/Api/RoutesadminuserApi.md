# Agdb\RoutesadminuserApi

All URIs are relative to http://localhost:3000, except if the operation defines another base path.

| Method | HTTP request | Description |
| ------------- | ------------- | ------------- |
| [**adminUserAdd()**](RoutesadminuserApi.md#adminUserAdd) | **POST** /api/v1/admin/user/{username}/add |  |
| [**adminUserChangePassword()**](RoutesadminuserApi.md#adminUserChangePassword) | **PUT** /api/v1/admin/user/{username}/change_password |  |
| [**adminUserList()**](RoutesadminuserApi.md#adminUserList) | **GET** /api/v1/admin/user/list |  |
| [**adminUserRemove()**](RoutesadminuserApi.md#adminUserRemove) | **DELETE** /api/v1/admin/user/{username}/remove |  |


## `adminUserAdd()`

```php
adminUserAdd($username, $user_credentials)
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadminuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$username = 'username_example'; // string | desired user name
$user_credentials = new \Agdb\Model\UserCredentials(); // \Agdb\Model\UserCredentials

try {
    $apiInstance->adminUserAdd($username, $user_credentials);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadminuserApi->adminUserAdd: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **username** | **string**| desired user name | |
| **user_credentials** | [**\Agdb\Model\UserCredentials**](../Model/UserCredentials.md)|  | |

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
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadminuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$username = 'username_example'; // string | user name
$user_credentials = new \Agdb\Model\UserCredentials(); // \Agdb\Model\UserCredentials

try {
    $apiInstance->adminUserChangePassword($username, $user_credentials);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadminuserApi->adminUserChangePassword: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **username** | **string**| user name | |
| **user_credentials** | [**\Agdb\Model\UserCredentials**](../Model/UserCredentials.md)|  | |

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
adminUserList(): \Agdb\Model\UserStatus[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadminuserApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);

try {
    $result = $apiInstance->adminUserList();
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling RoutesadminuserApi->adminUserList: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**\Agdb\Model\UserStatus[]**](../Model/UserStatus.md)

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
adminUserRemove($username): \Agdb\Model\UserStatus[]
```



### Example

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');


// Configure Bearer authorization: Token
$config = Agdb\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agdb\Api\RoutesadminuserApi(
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
    echo 'Exception when calling RoutesadminuserApi->adminUserRemove: ', $e->getMessage(), PHP_EOL;
}
```

### Parameters

| Name | Type | Description  | Notes |
| ------------- | ------------- | ------------- | ------------- |
| **username** | **string**| user name | |

### Return type

[**\Agdb\Model\UserStatus[]**](../Model/UserStatus.md)

### Authorization

[Token](../../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

[[Back to top]](#) [[Back to API list]](../../README.md#endpoints)
[[Back to Model list]](../../README.md#models)
[[Back to README]](../../README.md)
