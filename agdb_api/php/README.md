# OpenAPIClient-php

Agnesoft Graph Database Server


## Installation & Usage

### Requirements

PHP 8.1 and later.

### Composer

To install the bindings via [Composer](https://getcomposer.org/), add the following to `composer.json`:

```json
{
  "repositories": [
    {
      "type": "vcs",
      "url": "https://github.com/GIT_USER_ID/GIT_REPO_ID.git"
    }
  ],
  "require": {
    "GIT_USER_ID/GIT_REPO_ID": "*@dev"
  }
}
```

Then run `composer install`

### Manual Installation

Download the files and include `autoload.php`:

```php
<?php
require_once('/path/to/OpenAPIClient-php/vendor/autoload.php');
```

## Getting Started

Please follow the [installation procedure](#installation--usage) and then run the following:

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');



// Configure Bearer authorization: Token
$config = Agnesoft\AgdbApi\Configuration::getDefaultConfiguration()->setAccessToken('YOUR_ACCESS_TOKEN');


$apiInstance = new Agnesoft\AgdbApi\Api\AgdbApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client(),
    $config
);
$owner = 'owner_example'; // string | user name
$db = 'db_example'; // string | db name
$db_type = new \Agnesoft\AgdbApi\Model\\Agnesoft\AgdbApi\Model\DbKind(); // \Agnesoft\AgdbApi\Model\DbKind

try {
    $apiInstance->adminDbAdd($owner, $db, $db_type);
} catch (Exception $e) {
    echo 'Exception when calling AgdbApi->adminDbAdd: ', $e->getMessage(), PHP_EOL;
}

```

## API Endpoints

All URIs are relative to *http://localhost:3000*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*AgdbApi* | [**adminDbAdd**](docs/Api/AgdbApi.md#admindbadd) | **POST** /api/v1/admin/db/{owner}/{db}/add | 
*AgdbApi* | [**adminDbAudit**](docs/Api/AgdbApi.md#admindbaudit) | **GET** /api/v1/admin/db/{owner}/{db}/audit | 
*AgdbApi* | [**adminDbBackup**](docs/Api/AgdbApi.md#admindbbackup) | **POST** /api/v1/admin/db/{owner}/{db}/backup | 
*AgdbApi* | [**adminDbClear**](docs/Api/AgdbApi.md#admindbclear) | **POST** /api/v1/admin/db/{owner}/{db}/clear | 
*AgdbApi* | [**adminDbConvert**](docs/Api/AgdbApi.md#admindbconvert) | **POST** /api/v1/admin/db/{owner}/{db}/convert | 
*AgdbApi* | [**adminDbCopy**](docs/Api/AgdbApi.md#admindbcopy) | **POST** /api/v1/admin/db/{owner}/{db}/copy | 
*AgdbApi* | [**adminDbDelete**](docs/Api/AgdbApi.md#admindbdelete) | **DELETE** /api/v1/admin/db/{owner}/{db}/delete | 
*AgdbApi* | [**adminDbExec**](docs/Api/AgdbApi.md#admindbexec) | **POST** /api/v1/admin/db/{owner}/{db}/exec | 
*AgdbApi* | [**adminDbExecMut**](docs/Api/AgdbApi.md#admindbexecmut) | **POST** /api/v1/admin/db/{owner}/{db}/exec_mut | 
*AgdbApi* | [**adminDbList**](docs/Api/AgdbApi.md#admindblist) | **GET** /api/v1/admin/db/list | 
*AgdbApi* | [**adminDbOptimize**](docs/Api/AgdbApi.md#admindboptimize) | **POST** /api/v1/admin/db/{owner}/{db}/optimize | 
*AgdbApi* | [**adminDbRemove**](docs/Api/AgdbApi.md#admindbremove) | **DELETE** /api/v1/admin/db/{owner}/{db}/remove | 
*AgdbApi* | [**adminDbRename**](docs/Api/AgdbApi.md#admindbrename) | **POST** /api/v1/admin/db/{owner}/{db}/rename | 
*AgdbApi* | [**adminDbRestore**](docs/Api/AgdbApi.md#admindbrestore) | **POST** /api/v1/admin/db/{owner}/{db}/restore | 
*AgdbApi* | [**adminDbUserAdd**](docs/Api/AgdbApi.md#admindbuseradd) | **PUT** /api/v1/admin/db/{owner}/{db}/user/{username}/add | 
*AgdbApi* | [**adminDbUserList**](docs/Api/AgdbApi.md#admindbuserlist) | **GET** /api/v1/admin/db/{owner}/{db}/user/list | 
*AgdbApi* | [**adminDbUserRemove**](docs/Api/AgdbApi.md#admindbuserremove) | **DELETE** /api/v1/admin/db/{owner}/{db}/user/{username}/remove | 
*AgdbApi* | [**adminShutdown**](docs/Api/AgdbApi.md#adminshutdown) | **POST** /api/v1/admin/shutdown | 
*AgdbApi* | [**adminStatus**](docs/Api/AgdbApi.md#adminstatus) | **GET** /api/v1/admin/status | 
*AgdbApi* | [**adminUserAdd**](docs/Api/AgdbApi.md#adminuseradd) | **POST** /api/v1/admin/user/{username}/add | 
*AgdbApi* | [**adminUserChangePassword**](docs/Api/AgdbApi.md#adminuserchangepassword) | **PUT** /api/v1/admin/user/{username}/change_password | 
*AgdbApi* | [**adminUserDelete**](docs/Api/AgdbApi.md#adminuserdelete) | **DELETE** /api/v1/admin/user/{username}/delete | 
*AgdbApi* | [**adminUserList**](docs/Api/AgdbApi.md#adminuserlist) | **GET** /api/v1/admin/user/list | 
*AgdbApi* | [**adminUserLogout**](docs/Api/AgdbApi.md#adminuserlogout) | **POST** /api/v1/admin/user/{username}/logout | 
*AgdbApi* | [**adminUserLogoutAll**](docs/Api/AgdbApi.md#adminuserlogoutall) | **POST** /api/v1/admin/user/logout_all | 
*AgdbApi* | [**clusterAdminUserLogout**](docs/Api/AgdbApi.md#clusteradminuserlogout) | **POST** /api/v1/cluster/admin/user/{username}/logout | 
*AgdbApi* | [**clusterAdminUserLogoutAll**](docs/Api/AgdbApi.md#clusteradminuserlogoutall) | **POST** /api/v1/cluster/admin/user/logout_all | 
*AgdbApi* | [**clusterStatus**](docs/Api/AgdbApi.md#clusterstatus) | **GET** /api/v1/cluster/status | 
*AgdbApi* | [**clusterUserLogin**](docs/Api/AgdbApi.md#clusteruserlogin) | **POST** /api/v1/cluster/user/login | 
*AgdbApi* | [**clusterUserLogout**](docs/Api/AgdbApi.md#clusteruserlogout) | **POST** /api/v1/cluster/user/logout | 
*AgdbApi* | [**dbAdd**](docs/Api/AgdbApi.md#dbadd) | **POST** /api/v1/db/{owner}/{db}/add | 
*AgdbApi* | [**dbAudit**](docs/Api/AgdbApi.md#dbaudit) | **GET** /api/v1/db/{owner}/{db}/audit | 
*AgdbApi* | [**dbBackup**](docs/Api/AgdbApi.md#dbbackup) | **POST** /api/v1/db/{owner}/{db}/backup | 
*AgdbApi* | [**dbClear**](docs/Api/AgdbApi.md#dbclear) | **POST** /api/v1/db/{owner}/{db}/clear | 
*AgdbApi* | [**dbConvert**](docs/Api/AgdbApi.md#dbconvert) | **POST** /api/v1/db/{owner}/{db}/convert | 
*AgdbApi* | [**dbCopy**](docs/Api/AgdbApi.md#dbcopy) | **POST** /api/v1/db/{owner}/{db}/copy | 
*AgdbApi* | [**dbDelete**](docs/Api/AgdbApi.md#dbdelete) | **DELETE** /api/v1/db/{owner}/{db}/delete | 
*AgdbApi* | [**dbExec**](docs/Api/AgdbApi.md#dbexec) | **POST** /api/v1/db/{owner}/{db}/exec | 
*AgdbApi* | [**dbExecMut**](docs/Api/AgdbApi.md#dbexecmut) | **POST** /api/v1/db/{owner}/{db}/exec_mut | 
*AgdbApi* | [**dbList**](docs/Api/AgdbApi.md#dblist) | **GET** /api/v1/db/list | 
*AgdbApi* | [**dbOptimize**](docs/Api/AgdbApi.md#dboptimize) | **POST** /api/v1/db/{owner}/{db}/optimize | 
*AgdbApi* | [**dbRemove**](docs/Api/AgdbApi.md#dbremove) | **DELETE** /api/v1/db/{owner}/{db}/remove | 
*AgdbApi* | [**dbRename**](docs/Api/AgdbApi.md#dbrename) | **POST** /api/v1/db/{owner}/{db}/rename | 
*AgdbApi* | [**dbRestore**](docs/Api/AgdbApi.md#dbrestore) | **POST** /api/v1/db/{owner}/{db}/restore | 
*AgdbApi* | [**dbUserAdd**](docs/Api/AgdbApi.md#dbuseradd) | **PUT** /api/v1/db/{owner}/{db}/user/{username}/add | 
*AgdbApi* | [**dbUserList**](docs/Api/AgdbApi.md#dbuserlist) | **GET** /api/v1/db/{owner}/{db}/user/list | 
*AgdbApi* | [**dbUserRemove**](docs/Api/AgdbApi.md#dbuserremove) | **DELETE** /api/v1/db/{owner}/{db}/user/{username}/remove | 
*AgdbApi* | [**status**](docs/Api/AgdbApi.md#status) | **GET** /api/v1/status | 
*AgdbApi* | [**userChangePassword**](docs/Api/AgdbApi.md#userchangepassword) | **PUT** /api/v1/user/change_password | 
*AgdbApi* | [**userLogin**](docs/Api/AgdbApi.md#userlogin) | **POST** /api/v1/user/login | 
*AgdbApi* | [**userLogout**](docs/Api/AgdbApi.md#userlogout) | **POST** /api/v1/user/logout | 
*AgdbApi* | [**userStatus**](docs/Api/AgdbApi.md#userstatus) | **GET** /api/v1/user/status | 

## Models

- [AdminStatus](docs/Model/AdminStatus.md)
- [ChangePassword](docs/Model/ChangePassword.md)
- [ClusterStatus](docs/Model/ClusterStatus.md)
- [Comparison](docs/Model/Comparison.md)
- [ComparisonOneOf](docs/Model/ComparisonOneOf.md)
- [ComparisonOneOf1](docs/Model/ComparisonOneOf1.md)
- [ComparisonOneOf2](docs/Model/ComparisonOneOf2.md)
- [ComparisonOneOf3](docs/Model/ComparisonOneOf3.md)
- [ComparisonOneOf4](docs/Model/ComparisonOneOf4.md)
- [ComparisonOneOf5](docs/Model/ComparisonOneOf5.md)
- [ComparisonOneOf6](docs/Model/ComparisonOneOf6.md)
- [ComparisonOneOf7](docs/Model/ComparisonOneOf7.md)
- [ComparisonOneOf8](docs/Model/ComparisonOneOf8.md)
- [CountComparison](docs/Model/CountComparison.md)
- [CountComparisonOneOf](docs/Model/CountComparisonOneOf.md)
- [CountComparisonOneOf1](docs/Model/CountComparisonOneOf1.md)
- [CountComparisonOneOf2](docs/Model/CountComparisonOneOf2.md)
- [CountComparisonOneOf3](docs/Model/CountComparisonOneOf3.md)
- [CountComparisonOneOf4](docs/Model/CountComparisonOneOf4.md)
- [CountComparisonOneOf5](docs/Model/CountComparisonOneOf5.md)
- [DbElement](docs/Model/DbElement.md)
- [DbKeyOrder](docs/Model/DbKeyOrder.md)
- [DbKeyOrderOneOf](docs/Model/DbKeyOrderOneOf.md)
- [DbKeyOrderOneOf1](docs/Model/DbKeyOrderOneOf1.md)
- [DbKeyValue](docs/Model/DbKeyValue.md)
- [DbKind](docs/Model/DbKind.md)
- [DbResource](docs/Model/DbResource.md)
- [DbTypeParam](docs/Model/DbTypeParam.md)
- [DbUser](docs/Model/DbUser.md)
- [DbUserRole](docs/Model/DbUserRole.md)
- [DbUserRoleParam](docs/Model/DbUserRoleParam.md)
- [DbValue](docs/Model/DbValue.md)
- [DbValueOneOf](docs/Model/DbValueOneOf.md)
- [DbValueOneOf1](docs/Model/DbValueOneOf1.md)
- [DbValueOneOf2](docs/Model/DbValueOneOf2.md)
- [DbValueOneOf3](docs/Model/DbValueOneOf3.md)
- [DbValueOneOf4](docs/Model/DbValueOneOf4.md)
- [DbValueOneOf5](docs/Model/DbValueOneOf5.md)
- [DbValueOneOf6](docs/Model/DbValueOneOf6.md)
- [DbValueOneOf7](docs/Model/DbValueOneOf7.md)
- [DbValueOneOf8](docs/Model/DbValueOneOf8.md)
- [InsertAliasesQuery](docs/Model/InsertAliasesQuery.md)
- [InsertEdgesQuery](docs/Model/InsertEdgesQuery.md)
- [InsertIndexQuery](docs/Model/InsertIndexQuery.md)
- [InsertNodesQuery](docs/Model/InsertNodesQuery.md)
- [InsertValuesQuery](docs/Model/InsertValuesQuery.md)
- [KeyValueComparison](docs/Model/KeyValueComparison.md)
- [QueryAudit](docs/Model/QueryAudit.md)
- [QueryCondition](docs/Model/QueryCondition.md)
- [QueryConditionData](docs/Model/QueryConditionData.md)
- [QueryConditionDataOneOf](docs/Model/QueryConditionDataOneOf.md)
- [QueryConditionDataOneOf1](docs/Model/QueryConditionDataOneOf1.md)
- [QueryConditionDataOneOf2](docs/Model/QueryConditionDataOneOf2.md)
- [QueryConditionDataOneOf3](docs/Model/QueryConditionDataOneOf3.md)
- [QueryConditionDataOneOf4](docs/Model/QueryConditionDataOneOf4.md)
- [QueryConditionDataOneOf5](docs/Model/QueryConditionDataOneOf5.md)
- [QueryConditionDataOneOf6](docs/Model/QueryConditionDataOneOf6.md)
- [QueryConditionDataOneOf7](docs/Model/QueryConditionDataOneOf7.md)
- [QueryConditionLogic](docs/Model/QueryConditionLogic.md)
- [QueryConditionModifier](docs/Model/QueryConditionModifier.md)
- [QueryId](docs/Model/QueryId.md)
- [QueryIdOneOf](docs/Model/QueryIdOneOf.md)
- [QueryIdOneOf1](docs/Model/QueryIdOneOf1.md)
- [QueryIds](docs/Model/QueryIds.md)
- [QueryIdsOneOf](docs/Model/QueryIdsOneOf.md)
- [QueryIdsOneOf1](docs/Model/QueryIdsOneOf1.md)
- [QueryResult](docs/Model/QueryResult.md)
- [QueryType](docs/Model/QueryType.md)
- [QueryTypeOneOf](docs/Model/QueryTypeOneOf.md)
- [QueryTypeOneOf1](docs/Model/QueryTypeOneOf1.md)
- [QueryTypeOneOf10](docs/Model/QueryTypeOneOf10.md)
- [QueryTypeOneOf11](docs/Model/QueryTypeOneOf11.md)
- [QueryTypeOneOf12](docs/Model/QueryTypeOneOf12.md)
- [QueryTypeOneOf13](docs/Model/QueryTypeOneOf13.md)
- [QueryTypeOneOf14](docs/Model/QueryTypeOneOf14.md)
- [QueryTypeOneOf15](docs/Model/QueryTypeOneOf15.md)
- [QueryTypeOneOf16](docs/Model/QueryTypeOneOf16.md)
- [QueryTypeOneOf17](docs/Model/QueryTypeOneOf17.md)
- [QueryTypeOneOf2](docs/Model/QueryTypeOneOf2.md)
- [QueryTypeOneOf3](docs/Model/QueryTypeOneOf3.md)
- [QueryTypeOneOf4](docs/Model/QueryTypeOneOf4.md)
- [QueryTypeOneOf5](docs/Model/QueryTypeOneOf5.md)
- [QueryTypeOneOf6](docs/Model/QueryTypeOneOf6.md)
- [QueryTypeOneOf7](docs/Model/QueryTypeOneOf7.md)
- [QueryTypeOneOf8](docs/Model/QueryTypeOneOf8.md)
- [QueryTypeOneOf9](docs/Model/QueryTypeOneOf9.md)
- [QueryValues](docs/Model/QueryValues.md)
- [QueryValuesOneOf](docs/Model/QueryValuesOneOf.md)
- [QueryValuesOneOf1](docs/Model/QueryValuesOneOf1.md)
- [RemoveIndexQuery](docs/Model/RemoveIndexQuery.md)
- [RemoveQuery](docs/Model/RemoveQuery.md)
- [RemoveValuesQuery](docs/Model/RemoveValuesQuery.md)
- [SearchQuery](docs/Model/SearchQuery.md)
- [SearchQueryAlgorithm](docs/Model/SearchQueryAlgorithm.md)
- [SelectAliasesQuery](docs/Model/SelectAliasesQuery.md)
- [SelectEdgeCountQuery](docs/Model/SelectEdgeCountQuery.md)
- [SelectKeyCountQuery](docs/Model/SelectKeyCountQuery.md)
- [SelectKeysQuery](docs/Model/SelectKeysQuery.md)
- [SelectValuesQuery](docs/Model/SelectValuesQuery.md)
- [ServerDatabase](docs/Model/ServerDatabase.md)
- [ServerDatabaseAdminRename](docs/Model/ServerDatabaseAdminRename.md)
- [ServerDatabaseRename](docs/Model/ServerDatabaseRename.md)
- [ServerDatabaseResource](docs/Model/ServerDatabaseResource.md)
- [UserCredentials](docs/Model/UserCredentials.md)
- [UserLogin](docs/Model/UserLogin.md)
- [UserStatus](docs/Model/UserStatus.md)

## Authorization

Authentication schemes defined for the API:
### Token

- **Type**: Bearer authentication

## Tests

To run the tests, use:

```bash
composer install
vendor/bin/phpunit
```

## Author



## About this package

This PHP package is automatically generated by the [OpenAPI Generator](https://openapi-generator.tech) project:

- API version: `0.12.6`
    - Package version: `0.7.2`
    - Generator version: `7.18.0`
- Build package: `org.openapitools.codegen.languages.PhpClientCodegen`
