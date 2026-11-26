<?php
// Needed to load the libraries installed by composer
require "vendor/autoload.php";

use Agnesoft\AgdbApi\AgdbApi;
use Agnesoft\AgdbApi\CurlClient;
use Agnesoft\AgdbApi\QueryBuilder;

// Create the client using the native cURL transport.
$client = AgdbApi::new(new CurlClient(), "localhost:3000");

// Runs the status query against the database
// and throws if the server is not accessible.
$client->status();

// Login as server admin.
$client->user_login("admin", "admin");

// Create user "php_user1".
$client->admin_user_add("php_user1", "php_user1");

// Login as "php_user1".
$client->user_login("php_user1", "php_user1");

// Creates memory mapped database "db1" for user "php_user1".
$client->db_add("php_user1", "db1", "mapped");

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

    // :3: Find a node starting at the "users" node with specific username.
    QueryBuilder::select()
        ->search()
        ->from("users")
        ->where()
        ->key("username")
        ->value("user1")
        ->query(),
];

// Execute the queries.
$result = $client->db_exec_mut("php_user1", "db1", $queries);

// Print the result of the last query.
print_r($result[1][3]);

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
//                         "String": "password123"
//                     }
//                 }
//             ]
//         },
//     ],
//     "result": 1
// }
