<?php
use PHPUnit\Framework\TestCase;
use Agnesoft\AgdbApi\Model\Comparison;
use Agnesoft\AgdbApi\Model\CountComparison;
use Agnesoft\AgdbApi\Model\DbValue;
use Agnesoft\AgdbApi\ComparisonBuilder;
use Agnesoft\AgdbApi\CountComparisonBuilder;
use Agnesoft\AgdbApi\QueryBuilder;
use Agnesoft\AgdbApi\Model\QueryId;

class TNoDbId
{
    public mixed $value = 1;
}

class TDbId
{
    public mixed $db_id = 1;
    public int $value = 1;
}

final class QueryBuilderTest extends TestCase
{
    public function testToQueryId_QueryId(): void
    {
        $query1 = QueryBuilder::search()
            ->from(new QueryId(["id" => 1]))
            ->query();
        $query2 = QueryBuilder::search()->from(1)->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToQueryIds_array(): void
    {
        $query1 = QueryBuilder::select()->ids([])->query();
        $ids = $query1->getSelectValues()->getIds()->getIds();
        $this->assertEquals($ids, []);
    }

    public function testToQueryIds_QueryId(): void
    {
        $query1 = QueryBuilder::select()
            ->ids(new QueryId(["id" => 1]))
            ->query();
        $query2 = QueryBuilder::select()->ids(1)->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToQueryIds_SearchQuery(): void
    {
        $query1 = QueryBuilder::select()
            ->ids(QueryBuilder::search()->from(1)->query()->getSearch())
            ->query();
        $query2 = QueryBuilder::select()
            ->ids(QueryBuilder::search()->from(1)->query())
            ->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToDbValue_float(): void
    {
        $query1 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("float")
            ->value(ComparisonBuilder::Equal(1.1))
            ->query();
        $query2 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("float")
            ->value(ComparisonBuilder::Equal(new DbValue(["f64" => 1.1])))
            ->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToDbValue_bool(): void
    {
        $query1 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("bool")
            ->value(ComparisonBuilder::Equal(true))
            ->query();
        $query2 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("bool")
            ->value(ComparisonBuilder::Equal(new DbValue(["string" => "true"])))
            ->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToDbValue_array(): void
    {
        $query1 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("array")
            ->value(ComparisonBuilder::Equal([]))
            ->query();
        $query2 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("array")
            ->value(ComparisonBuilder::Equal(new DbValue(["vec_i64" => []])))
            ->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToDbValue_array_f64(): void
    {
        $query1 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("array_f64")
            ->value(ComparisonBuilder::Equal([1.1]))
            ->query();
        $query2 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("array_f64")
            ->value(ComparisonBuilder::Equal(new DbValue(["vec_f64" => [1.1]])))
            ->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testToDbValue_array_str(): void
    {
        $query1 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("array_str")
            ->value(ComparisonBuilder::Equal(["str"]))
            ->query();
        $query2 = QueryBuilder::search()
            ->from(1)
            ->where()
            ->key("array_str")
            ->value(
                ComparisonBuilder::Equal(new DbValue(["vec_str" => ["str"]]))
            )
            ->query();
        $json1 = $query1->jsonSerialize();
        $json2 = $query2->jsonSerialize();
        $this->assertEquals($json2, $json1);
    }

    public function testCountComparison(): void
    {
        $this->assertEquals(
            CountComparisonBuilder::Equal(1),
            new CountComparison(["equal" => 1])
        );
        $this->assertEquals(
            CountComparisonBuilder::GreaterThan(1),
            new CountComparison(["greater_than" => 1])
        );
        $this->assertEquals(
            CountComparisonBuilder::GreaterThanOrEqual(1),
            new CountComparison(["greater_than_or_equal" => 1])
        );
        $this->assertEquals(
            CountComparisonBuilder::LessThan(1),
            new CountComparison(["less_than" => 1])
        );
        $this->assertEquals(
            CountComparisonBuilder::LessThanOrEqual(1),
            new CountComparison(["less_than_or_equal" => 1])
        );
        $this->assertEquals(
            CountComparisonBuilder::NotEqual(1),
            new CountComparison(["not_equal" => 1])
        );
    }

    public function testComparison(): void
    {
        $this->assertEquals(
            ComparisonBuilder::Equal(1),
            new Comparison(["equal" => new DbValue(["i64" => 1])])
        );
        $this->assertEquals(
            ComparisonBuilder::GreaterThan(1),
            new Comparison(["greater_than" => new DbValue(["i64" => 1])])
        );
        $this->assertEquals(
            ComparisonBuilder::GreaterThanOrEqual(1),
            new Comparison([
                "greater_than_or_equal" => new DbValue(["i64" => 1]),
            ])
        );
        $this->assertEquals(
            ComparisonBuilder::LessThan(1),
            new Comparison(["less_than" => new DbValue(["i64" => 1])])
        );
        $this->assertEquals(
            ComparisonBuilder::LessThanOrEqual(1),
            new Comparison(["less_than_or_equal" => new DbValue(["i64" => 1])])
        );
        $this->assertEquals(
            ComparisonBuilder::NotEqual(1),
            new Comparison(["not_equal" => new DbValue(["i64" => 1])])
        );
    }

    public function testInsertElements_noDbId(): void
    {
        $query = QueryBuilder::insert()->element(new TNoDbId())->query();
        $id = $query->getInsertValues()->getIds()->getIds()[0]->getId();
        $this->assertEquals($id, 0);
    }

    public function testInsertElements_DbId(): void
    {
        $t = new TDbId();
        $t->db_id = 1;
        $query = QueryBuilder::insert()->element($t)->query();
        $id = $query->getInsertValues()->getIds()->getIds()[0]->getId();
        $this->assertEquals($id, 1);
    }
}
