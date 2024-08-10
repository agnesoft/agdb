
// GENERATED. DO NOT MODIFY AS ANY CHANGES WILL BE LOST.
// query_test_generator.js
<?php
use Agnesoft\Agdb\QueryBuilder;
use Agnesoft\Agdb\DbKeyOrderBuilder;
use Agnesoft\Agdb\CountComparisonBuilder;
use Agnesoft\Agdb\ComparisonBuilder;

class T
{
    public mixed $db_id = null;
    public string $value1 = "";
    public int $value2 = 0;
}

final class QueryTest extends \PHPUnit\Framework\TestCase
{
    private static $test_queries; // @phpstan-ignore missingType.property

    public static function setUpBeforeClass(): void
    {
        self::$test_queries = json_decode(
            (string) file_get_contents(
                "../../agdb_server/openapi/test_queries.json"
            )
        );
    }
    public function testQueryBuilder0(): void
    {
        $query = QueryBuilder::insert()->aliases("a")->ids(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[0][1], $json);
    }
    public function testQueryBuilder1(): void
    {
        $query = QueryBuilder::insert()->aliases("a")->ids("b")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[1][1], $json);
    }
    public function testQueryBuilder2(): void
    {
        $query = QueryBuilder::insert()
            ->aliases(["a", "b"])
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[2][1], $json);
    }
    public function testQueryBuilder3(): void
    {
        $query = QueryBuilder::insert()->edges()->from(1)->to(2)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[3][1], $json);
    }
    public function testQueryBuilder4(): void
    {
        $query = QueryBuilder::insert()->edges()->from("a")->to("b")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[4][1], $json);
    }
    public function testQueryBuilder5(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from("a")
            ->to([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[5][1], $json);
    }
    public function testQueryBuilder6(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from([1, 2])
            ->to([2, 3])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[6][1], $json);
    }
    public function testQueryBuilder7(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from([1, 2])
            ->to([2, 3])
            ->each()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[7][1], $json);
    }
    public function testQueryBuilder8(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from([1, 2])
            ->to([2, 3])
            ->each()
            ->values([["k" => 1], ["k" => 2]])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[8][1], $json);
    }
    public function testQueryBuilder9(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from([1, 2])
            ->to([2, 3])
            ->each()
            ->values_uniform(["k" => 1, 1 => 10])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[9][1], $json);
    }
    public function testQueryBuilder10(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from("a")
            ->to([1, 2])
            ->values([["k" => 1], ["k" => 2]])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[10][1], $json);
    }
    public function testQueryBuilder11(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from("a")
            ->to([1, 2])
            ->values_uniform(["k" => "v", 1 => 10])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[11][1], $json);
    }
    public function testQueryBuilder12(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from(QueryBuilder::search()->from("a")->where()->node()->query())
            ->to(QueryBuilder::search()->from("b")->where()->node()->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[12][1], $json);
    }
    public function testQueryBuilder13(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from(QueryBuilder::search()->from("a")->where()->node()->query())
            ->to(QueryBuilder::search()->from("b")->where()->node()->query())
            ->values([["k" => 1], ["k" => 2]])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[13][1], $json);
    }
    public function testQueryBuilder14(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->from(QueryBuilder::search()->from("a")->where()->node()->query())
            ->to(QueryBuilder::search()->from("b")->where()->node()->query())
            ->values_uniform(["k" => "v", 1 => 10])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[14][1], $json);
    }
    public function testQueryBuilder15(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->ids(-3)
            ->from(1)
            ->to(2)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[15][1], $json);
    }
    public function testQueryBuilder16(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->ids([-3, -4])
            ->from(1)
            ->to(2)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[16][1], $json);
    }
    public function testQueryBuilder17(): void
    {
        $query = QueryBuilder::insert()
            ->edges()
            ->ids(QueryBuilder::search()->from(1)->where()->edge()->query())
            ->from(1)
            ->to(2)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[17][1], $json);
    }
    public function testQueryBuilder18(): void
    {
        $query = QueryBuilder::insert()->index("key")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[18][1], $json);
    }
    public function testQueryBuilder19(): void
    {
        $query = QueryBuilder::insert()->nodes()->count(2)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[19][1], $json);
    }
    public function testQueryBuilder20(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->count(2)
            ->values_uniform(["k" => "v", 1 => 10])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[20][1], $json);
    }
    public function testQueryBuilder21(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->aliases(["a", "b"])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[21][1], $json);
    }
    public function testQueryBuilder22(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->aliases(["a", "b"])
            ->values([["k" => 1], ["k" => 2]])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[22][1], $json);
    }
    public function testQueryBuilder23(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->aliases(["a", "b"])
            ->values_uniform(["k" => "v", 1 => 10])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[23][1], $json);
    }
    public function testQueryBuilder24(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->values([["k" => 1], ["k" => 2]])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[24][1], $json);
    }
    public function testQueryBuilder25(): void
    {
        $query = QueryBuilder::insert()->nodes()->ids(1)->count(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[25][1], $json);
    }
    public function testQueryBuilder26(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->ids([1, 2])
            ->count(1)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[26][1], $json);
    }
    public function testQueryBuilder27(): void
    {
        $query = QueryBuilder::insert()->nodes()->ids("a")->count(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[27][1], $json);
    }
    public function testQueryBuilder28(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->ids("a")
            ->aliases("a")
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[28][1], $json);
    }
    public function testQueryBuilder29(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->ids(["a", "b"])
            ->count(1)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[29][1], $json);
    }
    public function testQueryBuilder30(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->ids([1, 2])
            ->values([["k" => "v"], [1 => 10]])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[30][1], $json);
    }
    public function testQueryBuilder31(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->ids([1, 2])
            ->values_uniform(["k" => "v", 1 => 10])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[31][1], $json);
    }
    public function testQueryBuilder32(): void
    {
        $query = QueryBuilder::insert()
            ->nodes()
            ->ids(QueryBuilder::search()->from(1)->query())
            ->count(1)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[32][1], $json);
    }
    public function testQueryBuilder33(): void
    {
        $query = QueryBuilder::insert()->element(new T())->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[33][1], $json);
    }
    public function testQueryBuilder34(): void
    {
        $query = QueryBuilder::insert()
            ->elements([new T(), new T()])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[34][1], $json);
    }
    public function testQueryBuilder35(): void
    {
        $query = QueryBuilder::insert()
            ->values([["k" => "v", 1 => 10], ["k" => 2]])
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[35][1], $json);
    }
    public function testQueryBuilder36(): void
    {
        $query = QueryBuilder::insert()
            ->values([["k" => "v", 1 => 10], ["k" => 2]])
            ->ids(QueryBuilder::search()->from("a")->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[36][1], $json);
    }
    public function testQueryBuilder37(): void
    {
        $query = QueryBuilder::insert()
            ->values_uniform(["k" => "v", 1 => 10])
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[37][1], $json);
    }
    public function testQueryBuilder38(): void
    {
        $query = QueryBuilder::insert()
            ->values_uniform(["k" => "v", 1 => 10])
            ->ids(QueryBuilder::search()->from("a")->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[38][1], $json);
    }
    public function testQueryBuilder39(): void
    {
        $query = QueryBuilder::remove()->aliases("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[39][1], $json);
    }
    public function testQueryBuilder40(): void
    {
        $query = QueryBuilder::remove()
            ->aliases(["a", "b"])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[40][1], $json);
    }
    public function testQueryBuilder41(): void
    {
        $query = QueryBuilder::remove()->ids(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[41][1], $json);
    }
    public function testQueryBuilder42(): void
    {
        $query = QueryBuilder::remove()->ids("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[42][1], $json);
    }
    public function testQueryBuilder43(): void
    {
        $query = QueryBuilder::remove()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[43][1], $json);
    }
    public function testQueryBuilder44(): void
    {
        $query = QueryBuilder::remove()
            ->ids(["a", "b"])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[44][1], $json);
    }
    public function testQueryBuilder45(): void
    {
        $query = QueryBuilder::remove()
            ->ids(QueryBuilder::search()->from("a")->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[45][1], $json);
    }
    public function testQueryBuilder46(): void
    {
        $query = QueryBuilder::remove()->index("key")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[46][1], $json);
    }
    public function testQueryBuilder47(): void
    {
        $query = QueryBuilder::remove()
            ->values(["k1", "k2"])
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[47][1], $json);
    }
    public function testQueryBuilder48(): void
    {
        $query = QueryBuilder::remove()
            ->values(["k1", "k2"])
            ->ids(QueryBuilder::search()->from("a")->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[48][1], $json);
    }
    public function testQueryBuilder49(): void
    {
        $query = QueryBuilder::select()
            ->aliases()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[49][1], $json);
    }
    public function testQueryBuilder50(): void
    {
        $query = QueryBuilder::select()
            ->aliases()
            ->ids(QueryBuilder::search()->from(1)->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[50][1], $json);
    }
    public function testQueryBuilder51(): void
    {
        $query = QueryBuilder::select()->aliases()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[51][1], $json);
    }
    public function testQueryBuilder52(): void
    {
        $query = QueryBuilder::select()
            ->edge_count()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[52][1], $json);
    }
    public function testQueryBuilder53(): void
    {
        $query = QueryBuilder::select()
            ->edge_count_from()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[53][1], $json);
    }
    public function testQueryBuilder54(): void
    {
        $query = QueryBuilder::select()
            ->edge_count_to()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[54][1], $json);
    }
    public function testQueryBuilder55(): void
    {
        $query = QueryBuilder::select()->ids("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[55][1], $json);
    }
    public function testQueryBuilder56(): void
    {
        $query = QueryBuilder::select()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[56][1], $json);
    }
    public function testQueryBuilder57(): void
    {
        $query = QueryBuilder::select()
            ->ids(QueryBuilder::search()->from(1)->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[57][1], $json);
    }
    public function testQueryBuilder58(): void
    {
        $query = QueryBuilder::select()->indexes()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[58][1], $json);
    }
    public function testQueryBuilder59(): void
    {
        $query = QueryBuilder::select()->keys()->ids("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[59][1], $json);
    }
    public function testQueryBuilder60(): void
    {
        $query = QueryBuilder::select()
            ->keys()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[60][1], $json);
    }
    public function testQueryBuilder61(): void
    {
        $query = QueryBuilder::select()
            ->keys()
            ->ids(QueryBuilder::search()->from(1)->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[61][1], $json);
    }
    public function testQueryBuilder62(): void
    {
        $query = QueryBuilder::select()->key_count()->ids("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[62][1], $json);
    }
    public function testQueryBuilder63(): void
    {
        $query = QueryBuilder::select()
            ->key_count()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[63][1], $json);
    }
    public function testQueryBuilder64(): void
    {
        $query = QueryBuilder::select()
            ->key_count()
            ->ids(QueryBuilder::search()->from(1)->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[64][1], $json);
    }
    public function testQueryBuilder65(): void
    {
        $query = QueryBuilder::select()->node_count()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[65][1], $json);
    }
    public function testQueryBuilder66(): void
    {
        $query = QueryBuilder::select()
            ->values(["k", "k2"])
            ->ids("a")
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[66][1], $json);
    }
    public function testQueryBuilder67(): void
    {
        $query = QueryBuilder::select()
            ->values(["k", "k2"])
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[67][1], $json);
    }
    public function testQueryBuilder68(): void
    {
        $query = QueryBuilder::select()
            ->values(["k", "k2"])
            ->ids(QueryBuilder::search()->from(1)->query())
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[68][1], $json);
    }
    public function testQueryBuilder69(): void
    {
        $query = QueryBuilder::search()->from("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[69][1], $json);
    }
    public function testQueryBuilder70(): void
    {
        $query = QueryBuilder::search()->to(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[70][1], $json);
    }
    public function testQueryBuilder71(): void
    {
        $query = QueryBuilder::search()->from("a")->to("b")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[71][1], $json);
    }
    public function testQueryBuilder72(): void
    {
        $query = QueryBuilder::search()->breadth_first()->from("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[72][1], $json);
    }
    public function testQueryBuilder73(): void
    {
        $query = QueryBuilder::search()->depth_first()->to(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[73][1], $json);
    }
    public function testQueryBuilder74(): void
    {
        $query = QueryBuilder::search()->depth_first()->from("a")->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[74][1], $json);
    }
    public function testQueryBuilder75(): void
    {
        $query = QueryBuilder::search()->elements()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[75][1], $json);
    }
    public function testQueryBuilder76(): void
    {
        $query = QueryBuilder::search()->index("age")->value(20)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[76][1], $json);
    }
    public function testQueryBuilder77(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->order_by([
                DbKeyOrderBuilder::Desc("age"),
                DbKeyOrderBuilder::Asc("name"),
            ])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[77][1], $json);
    }
    public function testQueryBuilder78(): void
    {
        $query = QueryBuilder::search()->from(1)->offset(10)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[78][1], $json);
    }
    public function testQueryBuilder79(): void
    {
        $query = QueryBuilder::search()->from(1)->limit(5)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[79][1], $json);
    }
    public function testQueryBuilder80(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->order_by([DbKeyOrderBuilder::Desc("k")])
            ->offset(10)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[80][1], $json);
    }
    public function testQueryBuilder81(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->order_by([DbKeyOrderBuilder::Desc("k")])
            ->limit(5)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[81][1], $json);
    }
    public function testQueryBuilder82(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->order_by([DbKeyOrderBuilder::Desc("k")])
            ->offset(10)
            ->limit(5)
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[82][1], $json);
    }
    public function testQueryBuilder83(): void
    {
        $query = QueryBuilder::search()->from(1)->offset(10)->limit(5)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[83][1], $json);
    }
    public function testQueryBuilder84(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->distance(CountComparisonBuilder::LessThan(3))
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[84][1], $json);
    }
    public function testQueryBuilder85(): void
    {
        $query = QueryBuilder::search()->from(1)->where()->edge()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[85][1], $json);
    }
    public function testQueryBuilder86(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->edge_count(CountComparisonBuilder::GreaterThan(2))
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[86][1], $json);
    }
    public function testQueryBuilder87(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->edge_count_from(CountComparisonBuilder::Equal(1))
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[87][1], $json);
    }
    public function testQueryBuilder88(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->edge_count_to(CountComparisonBuilder::NotEqual(1))
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[88][1], $json);
    }
    public function testQueryBuilder89(): void
    {
        $query = QueryBuilder::search()->from(1)->where()->node()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[89][1], $json);
    }
    public function testQueryBuilder90(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("k")
            ->value(ComparisonBuilder::Equal(1))
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[90][1], $json);
    }
    public function testQueryBuilder91(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->keys(["k1", "k2"])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[91][1], $json);
    }
    public function testQueryBuilder92(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->not()
            ->keys(["k1", "k2"])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[92][1], $json);
    }
    public function testQueryBuilder93(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[93][1], $json);
    }
    public function testQueryBuilder94(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->beyond()
            ->keys(["k"])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[94][1], $json);
    }
    public function testQueryBuilder95(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->not()
            ->ids([1, 2])
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[95][1], $json);
    }
    public function testQueryBuilder96(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->not_beyond()
            ->ids("a")
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[96][1], $json);
    }
    public function testQueryBuilder97(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->node()
            ->or()
            ->edge()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[97][1], $json);
    }
    public function testQueryBuilder98(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->node()
            ->and()
            ->distance(CountComparisonBuilder::GreaterThanOrEqual(3))
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[98][1], $json);
    }
    public function testQueryBuilder99(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->node()
            ->or()
            ->where()
            ->edge()
            ->and()
            ->key("k")
            ->value(ComparisonBuilder::Equal(1))
            ->end_where()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[99][1], $json);
    }
    public function testQueryBuilder100(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->node()
            ->or()
            ->where()
            ->edge()
            ->and()
            ->key("k")
            ->value(ComparisonBuilder::Contains(1))
            ->end_where()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[100][1], $json);
    }
    public function testQueryBuilder101(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->where()
            ->node()
            ->or()
            ->where()
            ->edge()
            ->and()
            ->key("k")
            ->value(ComparisonBuilder::Contains([1, 2]))
            ->end_where()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[101][1], $json);
    }
    public function testQueryBuilder102(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->order_by([DbKeyOrderBuilder::Asc("k")])
            ->where()
            ->node()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[102][1], $json);
    }
    public function testQueryBuilder103(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->limit(1)
            ->where()
            ->node()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[103][1], $json);
    }
    public function testQueryBuilder104(): void
    {
        $query = QueryBuilder::search()
            ->from(1)
            ->offset(1)
            ->where()
            ->node()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[104][1], $json);
    }
    public function testQueryBuilder105(): void
    {
        $query = QueryBuilder::search()->to(1)->offset(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[105][1], $json);
    }
    public function testQueryBuilder106(): void
    {
        $query = QueryBuilder::search()->to(1)->limit(1)->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[106][1], $json);
    }
    public function testQueryBuilder107(): void
    {
        $query = QueryBuilder::search()->to(1)->where()->node()->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[107][1], $json);
    }
    public function testQueryBuilder108(): void
    {
        $query = QueryBuilder::search()
            ->to(1)
            ->order_by([DbKeyOrderBuilder::Asc("k")])
            ->where()
            ->node()
            ->query();
        $json = $query->jsonSerialize();
        $this->assertEquals(self::$test_queries[108][1], $json);
    }
}

