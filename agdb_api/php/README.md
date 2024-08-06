# OpenAPIClient-php

Agnesoft Graph Database Server


## Installation & Usage

### Requirements

PHP 7.4 and later.
Should also work with PHP 8.0.

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

## API Endpoints

All URIs are relative to *http://localhost:3000*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*RoutesApi* | [**status**](docs/Api/RoutesApi.md#status) | **GET** /api/v1/status | 
*RoutesadminApi* | [**adminShutdown**](docs/Api/RoutesadminApi.md#adminshutdown) | **POST** /api/v1/admin/shutdown | 
*RoutesadmindbApi* | [**adminDbAdd**](docs/Api/RoutesadmindbApi.md#admindbadd) | **POST** /api/v1/admin/db/{owner}/{db}/add | 
*RoutesadmindbApi* | [**adminDbAudit**](docs/Api/RoutesadmindbApi.md#admindbaudit) | **GET** /api/v1/admin/db/{owner}/{db}/audit | 
*RoutesadmindbApi* | [**adminDbBackup**](docs/Api/RoutesadmindbApi.md#admindbbackup) | **POST** /api/v1/admin/db/{owner}/{db}/backup | 
*RoutesadmindbApi* | [**adminDbCopy**](docs/Api/RoutesadmindbApi.md#admindbcopy) | **POST** /api/v1/admin/db/{owner}/{db}/copy | 
*RoutesadmindbApi* | [**adminDbDelete**](docs/Api/RoutesadmindbApi.md#admindbdelete) | **DELETE** /api/v1/admin/db/{owner}/{db}/delete | 
*RoutesadmindbApi* | [**adminDbExec**](docs/Api/RoutesadmindbApi.md#admindbexec) | **POST** /api/v1/admin/db/{owner}/{db}/exec | 
*RoutesadmindbApi* | [**adminDbList**](docs/Api/RoutesadmindbApi.md#admindblist) | **GET** /api/v1/admin/db/list | 
*RoutesadmindbApi* | [**adminDbOptimize**](docs/Api/RoutesadmindbApi.md#admindboptimize) | **POST** /api/v1/admin/db/{owner}/{db}/optimize | 
*RoutesadmindbApi* | [**adminDbRemove**](docs/Api/RoutesadmindbApi.md#admindbremove) | **DELETE** /api/v1/admin/db/{owner}/{db}/remove | 
*RoutesadmindbApi* | [**adminDbRename**](docs/Api/RoutesadmindbApi.md#admindbrename) | **POST** /api/v1/admin/db/{owner}/{db}/rename | 
*RoutesadmindbApi* | [**adminDbRestore**](docs/Api/RoutesadmindbApi.md#admindbrestore) | **POST** /api/v1/db/admin/{owner}/{db}/restore | 
*RoutesadmindbuserApi* | [**adminDbUserAdd**](docs/Api/RoutesadmindbuserApi.md#admindbuseradd) | **PUT** /api/v1/admin/db/{owner}/{db}/user/{username}/add | 
*RoutesadmindbuserApi* | [**adminDbUserList**](docs/Api/RoutesadmindbuserApi.md#admindbuserlist) | **GET** /api/v1/admin/db/{owner}/{db}/user/list | 
*RoutesadmindbuserApi* | [**adminDbUserRemove**](docs/Api/RoutesadmindbuserApi.md#admindbuserremove) | **DELETE** /api/v1/admin/db/{owner}/{db}/user/{username}/remove | 
*RoutesadminuserApi* | [**adminUserAdd**](docs/Api/RoutesadminuserApi.md#adminuseradd) | **POST** /api/v1/admin/user/{username}/add | 
*RoutesadminuserApi* | [**adminUserChangePassword**](docs/Api/RoutesadminuserApi.md#adminuserchangepassword) | **PUT** /api/v1/admin/user/{username}/change_password | 
*RoutesadminuserApi* | [**adminUserList**](docs/Api/RoutesadminuserApi.md#adminuserlist) | **GET** /api/v1/admin/user/list | 
*RoutesadminuserApi* | [**adminUserRemove**](docs/Api/RoutesadminuserApi.md#adminuserremove) | **DELETE** /api/v1/admin/user/{username}/remove | 
*RoutesdbApi* | [**dbAdd**](docs/Api/RoutesdbApi.md#dbadd) | **POST** /api/v1/db/{owner}/{db}/add | 
*RoutesdbApi* | [**dbAudit**](docs/Api/RoutesdbApi.md#dbaudit) | **GET** /api/v1/db/{owner}/{db}/audit | 
*RoutesdbApi* | [**dbBackup**](docs/Api/RoutesdbApi.md#dbbackup) | **POST** /api/v1/db/{owner}/{db}/backup | 
*RoutesdbApi* | [**dbClear**](docs/Api/RoutesdbApi.md#dbclear) | **POST** /api/v1/db/{owner}/{db}/clear | 
*RoutesdbApi* | [**dbCopy**](docs/Api/RoutesdbApi.md#dbcopy) | **POST** /api/v1/db/{owner}/{db}/copy | 
*RoutesdbApi* | [**dbDelete**](docs/Api/RoutesdbApi.md#dbdelete) | **DELETE** /api/v1/db/{owner}/{db}/delete | 
*RoutesdbApi* | [**dbExec**](docs/Api/RoutesdbApi.md#dbexec) | **POST** /api/v1/db/{owner}/{db}/exec | 
*RoutesdbApi* | [**dbList**](docs/Api/RoutesdbApi.md#dblist) | **GET** /api/v1/db/list | 
*RoutesdbApi* | [**dbOptimize**](docs/Api/RoutesdbApi.md#dboptimize) | **POST** /api/v1/db/{owner}/{db}/optimize | 
*RoutesdbApi* | [**dbRemove**](docs/Api/RoutesdbApi.md#dbremove) | **DELETE** /api/v1/db/{owner}/{db}/remove | 
*RoutesdbApi* | [**dbRename**](docs/Api/RoutesdbApi.md#dbrename) | **POST** /api/v1/db/{owner}/{db}/rename | 
*RoutesdbApi* | [**dbRestore**](docs/Api/RoutesdbApi.md#dbrestore) | **POST** /api/v1/db/{owner}/{db}/restore | 
*RoutesdbuserApi* | [**dbUserAdd**](docs/Api/RoutesdbuserApi.md#dbuseradd) | **PUT** /api/v1/db/{owner}/{db}/user/{username}/add | 
*RoutesdbuserApi* | [**dbUserList**](docs/Api/RoutesdbuserApi.md#dbuserlist) | **GET** /api/v1/db/{owner}/{db}/user/list | 
*RoutesdbuserApi* | [**dbUserRemove**](docs/Api/RoutesdbuserApi.md#dbuserremove) | **POST** /api/v1/db/{owner}/{db}/user/{username}/remove | 
*RoutesuserApi* | [**userChangePassword**](docs/Api/RoutesuserApi.md#userchangepassword) | **PUT** /api/v1/user/change_password | 
*RoutesuserApi* | [**userLogin**](docs/Api/RoutesuserApi.md#userlogin) | **POST** /api/v1/user/login | 
*RoutesuserApi* | [**userLogout**](docs/Api/RoutesuserApi.md#userlogout) | **POST** /api/v1/user/logout | 

## Models

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
- [DbResource](docs/Model/DbResource.md)
- [DbType](docs/Model/DbType.md)
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
- [InsertNodesQuery](docs/Model/InsertNodesQuery.md)
- [InsertValuesQuery](docs/Model/InsertValuesQuery.md)
- [QueryAudit](docs/Model/QueryAudit.md)
- [QueryCondition](docs/Model/QueryCondition.md)
- [QueryConditionData](docs/Model/QueryConditionData.md)
- [QueryConditionDataOneOf](docs/Model/QueryConditionDataOneOf.md)
- [QueryConditionDataOneOf1](docs/Model/QueryConditionDataOneOf1.md)
- [QueryConditionDataOneOf2](docs/Model/QueryConditionDataOneOf2.md)
- [QueryConditionDataOneOf3](docs/Model/QueryConditionDataOneOf3.md)
- [QueryConditionDataOneOf4](docs/Model/QueryConditionDataOneOf4.md)
- [QueryConditionDataOneOf5](docs/Model/QueryConditionDataOneOf5.md)
- [QueryConditionDataOneOf5KeyValue](docs/Model/QueryConditionDataOneOf5KeyValue.md)
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
- [SearchQuery](docs/Model/SearchQuery.md)
- [SearchQueryAlgorithm](docs/Model/SearchQueryAlgorithm.md)
- [SelectEdgeCountQuery](docs/Model/SelectEdgeCountQuery.md)
- [SelectValuesQuery](docs/Model/SelectValuesQuery.md)
- [ServerDatabase](docs/Model/ServerDatabase.md)
- [ServerDatabaseRename](docs/Model/ServerDatabaseRename.md)
- [ServerDatabaseResource](docs/Model/ServerDatabaseResource.md)
- [StatusParams](docs/Model/StatusParams.md)
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

- API version: `0.7.2`
    - Package version: `0.7.2`
    - Generator version: `7.7.0`
- Build package: `org.openapitools.codegen.languages.PhpClientCodegen`
