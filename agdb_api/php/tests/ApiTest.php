<?php
use PHPUnit\Framework\TestCase;
use Agnesoft\AgdbApi\Model\DbElement;
use Agnesoft\AgdbApi\Model\DbType;
use Agnesoft\AgdbApi\Model\UserCredentials;
use Agnesoft\AgdbApi\QueryBuilder;
use Agnesoft\AgdbApi\Api\AgdbApi;
use Agnesoft\AgdbApi\Model\UserLogin;

class User
{
    public int $db_id = 0;
    public string $name;
    public int $age;
    public float $value;
    public bool $flag;
    public array $listInt;
    public array $listFloat;
    public array $listString;
}

final class ApiTest extends TestCase
{
    private static AgdbApi $client;

    public static function setUpBeforeClass(): void
    {
        new Agdb();
        $config = Agnesoft\AgdbApi\Configuration::getDefaultConfiguration();
        self::$client = new AgdbApi(new GuzzleHttp\Client(), $config);

        // Wait for server to be ready
        $attempts = 0;
        $maxAttempts = 10;
        $success = false;

        echo "Waiting for server to start..." . PHP_EOL;

        while ($attempts < $maxAttempts) {
            try {
                echo "Attempt " .
                    ($attempts + 1) .
                    "/$maxAttempts: Checking if server is ready..." .
                    PHP_EOL;
                self::$client->status();
                $success = true;
                echo "Server is ready!" . PHP_EOL;
                break;
            } catch (\Exception $e) {
                echo "Server not ready yet: " . $e->getMessage() . PHP_EOL;
                $attempts++;

                if ($attempts < $maxAttempts) {
                    echo "Waiting 2 seconds before next attempt..." . PHP_EOL;
                    sleep(2);
                }
            }
        }

        if (!$success) {
            throw new \RuntimeException(
                "Server failed to start after $maxAttempts attempts"
            );
        }
    }

    public function testInsertNodesAndEdges(): void
    {
        $token = self::$client->userLogin(
            new UserLogin(["username" => "admin", "password" => "admin"])
        );
        self::$client->getConfig()->setAccessToken($token);
        self::$client->adminUserAdd(
            "php_user1",
            new UserCredentials(["password" => "php_user1"])
        );
        $token = self::$client->userLogin(
            new UserLogin([
                "username" => "php_user1",
                "password" => "php_user1",
            ])
        );
        self::$client->getConfig()->setAccessToken($token);
        self::$client->dbAdd("php_user1", "db1", DbType::MAPPED); // @phpstan-ignore argument.type
        $res = self::$client->dbExecMut("php_user1", "db1", [
            QueryBuilder::insert()->nodes()->aliases("root")->query(),
            QueryBuilder::insert()->nodes()->count(2)->query(),
            QueryBuilder::insert()->edges()->from("root")->to(":1")->query(),
        ]);
        $this->assertEquals(3, count($res));
    }

    public function testInsertReadElements(): void
    {
        $token = self::$client->userLogin(
            new UserLogin(["username" => "admin", "password" => "admin"])
        );
        self::$client->getConfig()->setAccessToken($token);
        self::$client->adminUserAdd(
            "php_user2",
            new UserCredentials(["password" => "php_user2"])
        );
        $token = self::$client->userLogin(
            new UserLogin([
                "username" => "php_user2",
                "password" => "php_user2",
            ])
        );
        self::$client->getConfig()->setAccessToken($token);
        self::$client->dbAdd("php_user2", "db1", DbType::MAPPED); // @phpstan-ignore argument.type

        $person1 = new User();
        $person1->name = "John";
        $person1->age = 30;
        $person1->value = 1.1;
        $person1->flag = true;
        $person1->listInt = [1, 2, 2];
        $person1->listFloat = [-1.111, -3.333, 2.0];
        $person1->listString = ["hello", "world"];

        $person2 = new User();
        $person2->name = "Jane";
        $person2->age = 25;
        $person2->value = -2.3;
        $person2->flag = false;
        $person2->listInt = [3, 2, 1];
        $person2->listFloat = [3.0, 2.2, 1.785];
        $person2->listString = ["a", "b", "c"];

        $res = self::$client->dbExecMut("php_user2", "db1", [
            QueryBuilder::insert()
                ->elements([$person1, $person2])
                ->query(),
            QueryBuilder::select()->ids(":0")->query(),
        ]);

        $persons = Agdb::try_into("User", $res[1]);
        $person1->db_id = 1;
        $person2->db_id = 2;

        $this->assertEquals([$person1, $person2], $persons);
    }

    public function testSearch(): void
    {
        $token = self::$client->userLogin(
            new UserLogin(["username" => "admin", "password" => "admin"])
        );
        self::$client->getConfig()->setAccessToken($token);
        self::$client->adminUserAdd(
            "php_user3",
            new UserCredentials(["password" => "php_user3"])
        );
        $token = self::$client->userLogin(
            new UserLogin([
                "username" => "php_user3",
                "password" => "php_user3",
            ])
        );
        self::$client->getConfig()->setAccessToken($token);
        self::$client->dbAdd("php_user3", "db1", DbType::MAPPED); // @phpstan-ignore argument.type
        $res = self::$client->dbExecMut("php_user3", "db1", [
            QueryBuilder::insert()->nodes()->count(1)->query(),
            QueryBuilder::insert()->nodes()->count(1)->query(),
            QueryBuilder::insert()->edges()->from(":0")->to(":1")->query(),
            QueryBuilder::search()->from(":0")->to(":1")->query(),
        ]);
        $this->assertEquals(4, count($res));
        $this->assertEquals(3, $res[3]->getResult());
        $this->assertEquals(
            [
                new DbElement([
                    "id" => 1,
                    "from" => null,
                    "to" => null,
                    "values" => [],
                ]),
                new DbElement([
                    "id" => -3,
                    "from" => 1,
                    "to" => 2,
                    "values" => [],
                ]),
                new DbElement([
                    "id" => 2,
                    "from" => null,
                    "to" => null,
                    "values" => [],
                ]),
            ],
            $res[3]->getElements()
        );
    }
}
