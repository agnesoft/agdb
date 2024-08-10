<?php
use Agdb\Model\DbElement;
use Agdb\Model\DbType;
use Agdb\Model\UserCredentials;
use Agnesoft\Agdb\QueryBuilder;
use PHPUnit\Framework\TestCase;
use Agdb\Api\AgdbApi;
use Agdb\Model\UserLogin;

class Person
{
    public mixed $db_id = null;
    public string $name;
    public int $age;
    public array $list;
}

final class ApiTest extends TestCase
{
    private static AgdbApi $client;

    public static function setUpBeforeClass(): void
    {
        $config = Agdb\Configuration::getDefaultConfiguration();
        self::$client = new AgdbApi(new GuzzleHttp\Client(), $config);
    }

    public function testStatus(): void
    {
        $response = self::$client->status(false);
        $this->assertIsArray($response);
    }

    public function testInsertNodesAndEdges(): void
    {
        $token = self::$client->userLogin(new UserLogin(['username' => 'admin', 'password' => 'admin']));
        self::$client->getConfig()->setAccessToken($token);
        self::$client->adminUserAdd('php_user1', new UserCredentials(['password' => 'php_user1']));
        $token = self::$client->userLogin(new UserLogin(['username' => 'php_user1', 'password' => 'php_user1']));
        self::$client->getConfig()->setAccessToken($token);
        self::$client->dbAdd('php_user1', 'db1', DbType::MAPPED); // @phpstan-ignore argument.type
        $res = self::$client->dbExec('php_user1', 'db1', [
            QueryBuilder::insert()->nodes()->aliases('root')->query(),
            QueryBuilder::insert()->nodes()->count(2)->query(),
            QueryBuilder::insert()->edges()->from('root')->to(':1')->query(),
        ]);
        $this->assertEquals(3, count($res));
    }

    public function testInsertReadElements(): void
    {
        $token = self::$client->userLogin(new UserLogin(['username' => 'admin', 'password' => 'admin']));
        self::$client->getConfig()->setAccessToken($token);
        self::$client->adminUserAdd('php_user2', new UserCredentials(['password' => 'php_user2']));
        $token = self::$client->userLogin(new UserLogin(['username' => 'php_user2', 'password' => 'php_user2']));
        self::$client->getConfig()->setAccessToken($token);
        self::$client->dbAdd('php_user2', 'db1', DbType::MAPPED); // @phpstan-ignore argument.type

        $person1 = new Person();
        $person1->name = 'John';
        $person1->age = 30;
        $person1->list = [1, 2, 2];

        $person2 = new Person();
        $person2->name = 'Jane';
        $person2->age = 25;
        $person2->list = [3, 2, 1];

        $res = self::$client->dbExec('php_user2', 'db1', [
            QueryBuilder::insert()->elements([$person1, $person2])->query(),
            QueryBuilder::select()->ids(':0')->query(),
        ]);

        $persons = [];

        foreach ($res[1]->getElements() as $element) {
            $person = new Person();

            foreach ($element->getValues() as $kv) {
                if ($kv->getKey()->getString() === 'name') {
                    $person->name = $kv->getValue()->getString();
                } elseif ($kv->getKey()->getString() === 'age') {
                    $person->age = $kv->getValue()->getI64();
                } elseif ($kv->getKey()->getString() === 'list') {
                    $person->list = $kv->getValue()->getVecI64();
                }
            }

            $persons[] = $person;
        }

        $this->assertEquals([$person1, $person2], $persons);
    }

    public function testSearch(): void
    {
        $token = self::$client->userLogin(new UserLogin(['username' => 'admin', 'password' => 'admin']));
        self::$client->getConfig()->setAccessToken($token);
        self::$client->adminUserAdd('php_user3', new UserCredentials(['password' => 'php_user3']));
        $token = self::$client->userLogin(new UserLogin(['username' => 'php_user3', 'password' => 'php_user3']));
        self::$client->getConfig()->setAccessToken($token);
        self::$client->dbAdd('php_user3', 'db1', DbType::MAPPED); // @phpstan-ignore argument.type
        $res = self::$client->dbExec('php_user3', 'db1', [
            QueryBuilder::insert()->nodes()->count(1)->query(),
            QueryBuilder::insert()->nodes()->count(1)->query(),
            QueryBuilder::insert()->edges()->from(":0")->to(":1")->query(),
            QueryBuilder::search()->from(":0")->to(":1")->query(),
        ]);
        $this->assertEquals(4, count($res));
        $this->assertEquals(3, $res[3]->getResult());
        $this->assertEquals([
            new DbElement(['id' => 1, 'from' => null, 'to' => null, 'values' => []]),
            new DbElement(['id' => -3, 'from' => 1, 'to' => 2, 'values' => []]),
            new DbElement(['id' => 2, 'from' => null, 'to' => null, 'values' => []]),

        ], $res[3]->getElements());
    }
}
