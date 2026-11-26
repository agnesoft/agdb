<?php

/**
 * Test harness for the route tests: the PHP equivalent of the TS
 * `test_server.ts`.  Provides `TestServer`, `TestCluster` and the name
 * generators the generated `ApiTest` calls.
 *
 * The environment variables are set by `agdb_harness` which starts the
 * server (and optionally a cluster) before running the test command.
 */

namespace Agnesoft\AgdbApi;

require_once __DIR__ . "/../src/types.php";
require_once __DIR__ . "/../src/query_builder.php";
require_once __DIR__ . "/../src/api.php";
require_once __DIR__ . "/../src/curl_client.php";

/** Shared monotonic counter for unique names across a test run. */
class Counter
{
    private static int $value = 0;

    public static function next(): int
    {
        return ++self::$value;
    }
}

function env(string $name): string
{
    $value = getenv($name);

    if ($value === false || $value === "") {
        throw new \RuntimeException(
            "$name is not set: run the tests with agdb_harness"
        );
    }

    return $value;
}

/** The name of the running test, used as the user-agent of its clients. */
function test_agent_name(): string
{
    return "agdb_api_test";
}

function api_with_agent(string $address, string $agent): AgdbApi
{
    return AgdbApi::new(new CurlClient($agent), $address);
}

function api_for_test(string $address): AgdbApi
{
    return api_with_agent($address, test_agent_name());
}

function next_user_name(): string
{
    return "db_user" . getmypid() . "_" . Counter::next();
}

function next_db_name(): string
{
    return "db" . getmypid() . "_" . Counter::next();
}

/**
 * The `test_server` module of the agdb_server tests the route tests are
 * generated from: the server the `agdb_harness` starts for the whole run,
 * named in its environment variables.
 */
class TestServer
{
    public function __construct(public string $data_dir, public AgdbApi $api) {}

    public static function new(): TestServer
    {
        return new TestServer(
            env("AGDB_SERVER_DATA_DIR"),
            api_for_test(env("AGDB_SERVER_ADDRESS"))
        );
    }
}

/** The leader is the first node. */
class TestCluster
{
    public function __construct(
        public array $addresses,
        public array $data_dirs
    ) {}

    public static function new(): TestCluster
    {
        return new TestCluster(
            explode(",", env("AGDB_CLUSTER_ADDRESSES")),
            explode(",", env("AGDB_CLUSTER_DATA_DIRS"))
        );
    }

    public function leader(): AgdbApi
    {
        return api_for_test($this->addresses[0]);
    }

    public function follower(): AgdbApi
    {
        return api_for_test($this->addresses[1]);
    }
}
