<?php
// Needed to load the libraries installed by composer
require "vendor/autoload.php";

use Agnesoft\AgdbApi\Api\AgdbApi;
use Agnesoft\AgdbApi\Model\DbType;
use Agnesoft\AgdbApi\Model\UserLogin;
use Agnesoft\AgdbApi\Model\UserCredentials;
use Agnesoft\AgdbApi\QueryBuilder;
use Agnesoft\AgdbApi\ComparisonBuilder;

// Default config will look for the server at http://localhost:3000
$config = Agnesoft\AgdbApi\Configuration::getDefaultConfiguration();

// Using GuzzleHttp client. You can use any other such as Symfony.
$client = new AgdbApi(new GuzzleHttp\Client(), $config);

// Runs the status query against the database
// and throws if the server is not accessible.
$client->status();

// Login as server admin.
$token = $client->userLogin(
    new UserLogin(["username" => "admin", "password" => "admin"])
);
$client->getConfig()->setAccessToken($token);

// Creat user "php_user1".
$client->adminUserAdd(
    "php_user1",
    new UserCredentials(["password" => "php_user1"])
);

// Login as "php_user1".
$token = $client->userLogin(
    new UserLogin([
        "username" => "php_user1",
        "password" => "php_user1",
    ])
);
$client->getConfig()->setAccessToken($token);

// Creates memory mapped database "db1" for user "php_user1".
$client->dbAdd("php_user1", "db1", DbType::MAPPED); // @phpstan-ignore argument.type

// Prepare the queries to be executed on the remote database.
$queries = [
    // :0: Inserts a root node aliased "users".
    QueryBuilder::insert()
        ->nodes()
        ->aliases(["users"])
        ->query(),

    // :1: Inserts more nodes with some data.
    QueryBuilder::insert()
        ->nodes()
        ->values([
            [
                "username" => "user1",
                "password" => "password123",
            ],
            [
                "username" => "user2",
                "password" => "password456",
            ],
        ])
        ->query(),

    // :2: Connect the root to the inserted nodes with edges referencing both from previous queries.
    QueryBuilder::insert()->edges()->from(":0")->to(":1")->query(),

    // :3: Find a node starting at the "users" node (could also be ":0" in this instance) with specific username.
    QueryBuilder::select()
        ->search()
        ->from("users")
        ->where()
        ->key("username")
        ->value(ComparisonBuilder::Equal("user1"))
        ->query(),
];

// Execute the queries.
$result = $client->dbExec("php_user1", "db1", $queries);

// Print the result of the last query.
printf($result[3]);

// {
//     "elements": [
//         {
//             "from": null,
//             "id": 3,
//             "to": null,
//             "values": [
//                 {
//                     "key": {
//                         "String": "username"
//                     },
//                     "value": {
//                         "String": "user1"
//                     }
//                 },
//                 {
//                     "key": {
//                         "String": "password"
//                     },
//                     "value": {
//                         "String": "password456"
//                     }
//                 }
//             ]
//         },
//     ],
//     "result": 1
// }

