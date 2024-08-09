<?php

namespace Agnesoft\Agdb;

use stdClass;
use Agdb\Model\CountComparison;
use Agdb\Model\QueryConditionLogic;
use Agdb\Model\QueryConditionModifier;
use Agdb\Model\Comparison;
use Agdb\Model\QueryCondition;
use Agdb\Model\QueryConditionData;
use Agdb\Model\DbKeyValue;
use Agdb\Model\DbValue;
use Agdb\Model\InsertAliasesQuery;
use Agdb\Model\InsertEdgesQuery;
use Agdb\Model\InsertNodesQuery;
use Agdb\Model\InsertValuesQuery;
use Agdb\Model\QueryId;
use Agdb\Model\QueryIds;
use Agdb\Model\QueryType;
use Agdb\Model\QueryValues;
use Agdb\Model\SearchQuery;
use Agdb\Model\SelectValuesQuery;
use Agdb\Model\SelectEdgeCountQuery;
use Agdb\Model\SearchQueryAlgorithm;
use Agdb\Model\QueryConditionDataOneOf5KeyValue;
use Agdb\Model\DbKeyOrder;

class CountComparisonBuilder
{
    public static function Equal(int $value): CountComparison
    {
        return new CountComparison(['equal' => $value]);
    }

    public static function GreaterThan(int $value): CountComparison
    {
        return new CountComparison(['greater_than' => $value]);
    }

    public static function GreaterThanOrEqual(int $value): CountComparison
    {
        return new CountComparison(['greater_than_or_equal' => $value]);
    }

    public static function LessThan(int $value): CountComparison
    {
        return new CountComparison(['less_than' => $value]);
    }

    public static function LessThanOrEqual(int $value): CountComparison
    {
        return new CountComparison(['less_than_or_equal' => $value]);
    }

    public static function NotEqual(int $value): CountComparison
    {
        return new CountComparison(['not_equal' => $value]);
    }
}

class ComparisonBuilder
{
    public static function Contains(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['equal' => to_db_value($value)]);
    }

    public static function Equal(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['equal' => to_db_value($value)]);
    }

    public static function GreaterThan(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['greater_than' => to_db_value($value)]);
    }

    public static function GreaterThanOrEqual(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['greater_than_or_equal' => to_db_value($value)]);
    }

    public static function LessThan(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['less_than' => to_db_value($value)]);
    }

    public static function LessThanOrEqual(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['less_than_or_equal' => to_db_value($value)]);
    }

    public static function NotEqual(bool|int|float|string|array|DbValue $value): Comparison
    {
        return new Comparison(['not_equal' => to_db_value($value)]);
    }
}

class DbKeyOrderBuilder
{
    public static function Asc(int|float|string|array|DbValue $value): DbKeyOrder
    {
        return new DbKeyOrder(['asc' => to_db_value($value)]);
    }

    public static function Desc(int|float|string|array|DbValue $value): DbKeyOrder
    {
        return new DbKeyOrder(['desc' => to_db_value($value)]);
    }
}

function to_query_id(string|int|QueryId $id): QueryId
{
    if (is_string($id)) {
        return new QueryId(['alias' => $id]);
    }

    if (is_int($id)) {
        return new QueryId(['id' => $id]);
    }

    if (get_class($id) === QueryId::class) {
        return $id;
    }

    throw new \InvalidArgumentException('Unsupported $id type', 1);
}

function to_query_ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): QueryIds
{
    if (is_string($ids)) {
        return new QueryIds(['ids' => [new QueryId(['alias' => $ids])]]);
    }

    if (is_int($ids)) {
        return new QueryIds(['ids' => [new QueryId(['id' => $ids])]]);
    }

    if (is_array($ids)) {
        if (count($ids) !== 0) {
            if (is_int($ids[0])) {
                return new QueryIds(['ids' => array_map(fn($id) => new QueryId(['id' => $id]), $ids)]);
            } else {
                return new QueryIds(['ids' => array_map(fn($id) => new QueryId(['alias' => $id]), $ids)]);
            }
        }
    }

    if (get_class($ids) === QueryId::class) {
        return new QueryIds(['ids' => [$ids]]);
    }

    if (get_class($ids) === SearchQuery::class) {
        return new QueryIds(['search' => $ids]);
    }

    if (get_class($ids) === QueryIds::class) {
        return $ids;
    }

    throw new \InvalidArgumentException('Unsupported $ids type', 1);
}

function to_db_value(bool|int|float|string|array|DbValue $value): DbValue
{
    if (is_string($value)) {
        return new DbValue(['string' => $value]);
    }

    if (is_int($value)) {
        return new DbValue(['int64' => $value]);
    }

    if (is_float($value)) {
        return new DbValue(['f64' => $value]);
    }

    if (is_bool($value)) {
        return new DbValue(['string' => $value ? 'true' : 'false']);
    }

    if (is_array($value)) {
        if (count($value) === 0) {
            return new DbValue(['vec_i64' => []]);
        } else {
            if (is_int($value[0])) {
                return new DbValue(['vec_i64' => $value]);
            } else if (is_float($value[0])) {
                return new DbValue(['vec_f64' => $value]);
            } else {
                return new DbValue(['vec_str' => $value]);
            }
        }
    }

    if (get_class($value) === DbValue::class) {
        return $value;
    }

    throw new \InvalidArgumentException('Unsupported $value type', 1);
}

function to_db_keys(array $data): array
{
    $keys = [];

    foreach ($data as $key) {
        $keys[] = to_db_value($key);
    }

    return $keys;

}

function to_multi_values(array $data): QueryValues
{
    $values = [];

    foreach ($data as $element) {
        $element_values = [];

        foreach ($element as $key => $value) {
            $element_values[] = new DbKeyValue(['key' => to_db_value($key), 'value' => to_db_value($value)]);
        }

        $values[] = $element_values;
    }

    return new QueryValues(['multi' => $values]);
}

function to_single_values(array $data): QueryValues
{
    $values = [];

    foreach ($data as $key => $value) {
        $values[] = new DbKeyValue(['key' => to_db_value($key), 'value' => to_db_value($value)]);
    }

    return new QueryValues(['single' => $values]);
}

class InsertAliasesIdsBuilder
{
    private InsertAliasesQuery $data;

    function __construct(InsertAliasesQuery $data)
    {
        $this->data = $data;
    }

    function query(): QueryType
    {
        return new QueryType(['insert_alias' => $this->data]);
    }
}

class InsertAliasesBuilder
{
    private InsertAliasesQuery $data;

    function __construct(array $names)
    {
        $this->data = new InsertAliasesQuery(['aliases' => $names, 'ids' => []]);
    }

    function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertAliasesIdsBuilder
    {
        $this->data->setIds(to_query_ids($ids));
        return new InsertAliasesIdsBuilder($this->data);
    }
}

class InsertValuesIdsBuilder
{
    private InsertValuesQuery $data;

    function __construct(InsertValuesQuery $data)
    {
        $this->data = $data;
    }

    function query(): QueryType
    {
        return new QueryType(['insert_values' => $this->data]);
    }
}

class InsertEdgesToEachBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function values($values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function values_uniform($values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_edges' => $this->data]);
    }
}

class InsertEdgesValuesBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_edges' => $this->data]);
    }
}

class InsertEdgesToBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function each(): InsertEdgesToEachBuilder
    {
        $this->data->setEach(true);
        return new InsertEdgesToEachBuilder($this->data);
    }

    public function values($values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function values_uniform($values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_edges' => $this->data]);
    }
}

class InsertEdgesFromBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function to(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertEdgesToBuilder
    {
        $this->data->setTo(to_query_ids($ids));
        return new InsertEdgesToBuilder($this->data);
    }
}

class InsertEdgesIdsBuilder
{
    private QueryIds $data;

    public function __construct(QueryIds $ids)
    {
        $this->data = $ids;
    }

    public function from(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertEdgesFromBuilder
    {
        return new InsertEdgesFromBuilder(new InsertEdgesQuery(['ids' => $this->data]));
    }
}

class InsertEdgesBuilder
{
    public function from(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertEdgesFromBuilder
    {
        return new InsertEdgesFromBuilder(new InsertEdgesQuery(['from' => to_query_ids($ids)]));
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertEdgesIdsBuilder
    {
        return new InsertEdgesIdsBuilder(to_query_ids($ids));
    }
}

class InsertNodesValuesBuilder
{
    private InsertNodesQuery $data;

    public function __construct(InsertNodesQuery $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_nodes' => $this->data]);
    }
}

class InsertNodesAliasesBuilder
{
    private InsertNodesQuery $data;

    public function __construct(InsertNodesQuery $data)
    {
        $this->data = $data;
    }

    public function values($values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function values_uniform($values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_nodes' => $this->data]);
    }
}

class InsertNodesCountBuilder
{
    private InsertNodesQuery $data;

    public function __construct(InsertNodesQuery $data)
    {
        $this->data = $data;
    }

    public function values_uniform($values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_nodes' => $this->data]);
    }
}

class InsertNodesIdsBuilder
{
    private InsertNodesQuery $data;

    public function __construct(InsertNodesQuery $data)
    {
        $this->data = $data;
    }

    public function aliases(string|array $names): InsertNodesAliasesBuilder
    {
        $this->data->setAliases(is_array($names) ? $names : array($names));
        return new InsertNodesAliasesBuilder($this->data);
    }

    public function count(int $count): InsertNodesCountBuilder
    {
        $this->data->setCount($count);
        return new InsertNodesCountBuilder($this->data);
    }

    public function values($values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function values_uniform($values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }
}

class InsertNodesBuilder
{
    public function aliases(string|array $names): InsertNodesAliasesBuilder
    {
        return new InsertNodesAliasesBuilder(new InsertNodesQuery(['aliases' => is_array($names) ? $names : array($names)]));
    }

    public function count(int $count): InsertNodesCountBuilder
    {
        return new InsertNodesCountBuilder(new InsertNodesQuery(['count' => $count]));
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertNodesIdsBuilder
    {
        return new InsertNodesIdsBuilder(new InsertNodesQuery(['ids' => to_query_ids($ids)]));
    }

    public function values($values): InsertNodesValuesBuilder
    {
        return new InsertNodesValuesBuilder(new InsertNodesQuery(['values' => to_multi_values($values)]));
    }
}

class InsertIndexBuilder
{
    private DbValue $data;

    public function __construct(DbValue $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['insert_index' => $this->data]);
    }
}

class InsertValuesBuilder
{
    private QueryValues $data;

    public function __construct(QueryValues $data)
    {
        $this->data = $data;
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): InsertValuesIdsBuilder
    {
        return new InsertValuesIdsBuilder(new InsertValuesQuery(['values' => $this->data, 'ids' => to_query_ids($ids)]));
    }
}

class InsertBuilder
{
    public function aliases(string|array $names): InsertAliasesBuilder
    {
        return new InsertAliasesBuilder(is_array($names) ? $names : array($names));
    }

    public function element(mixed $elem): InsertValuesIdsBuilder
    {
        return $this->elements(array($elem));
    }

    public function elements(array $elems): InsertValuesIdsBuilder
    {
        $data = new InsertValuesQuery();

        foreach ($elems as $elem) {
            if (property_exists($elem, 'db_id')) {
                $values = [];

                foreach ($elem as $key => $value) {
                    if ($key === 'db_id') {
                        if (is_null($key) || $value === 0) {
                            $data->getIds()->getIds()[] = new QueryId(['id' => 0]);
                        } else if (is_int($value)) {
                            $data->getIds()->getIds()[] = new QueryId(['id' => $value]);
                        } else {
                            $data->getIds()->getIds()[] = new QueryId(['alias' => $value]);
                        }
                    } else {
                        $values[] = new DbKeyValue(['key' => new DbValue(['string' => $key]), 'value' => to_db_value($value)]);
                    }
                }

                $data->getValues()->getMulti()[] = $values;
            }
        }

        return new InsertValuesIdsBuilder($data);
    }

    public function edges(): InsertEdgesBuilder
    {
        return new InsertEdgesBuilder();
    }

    public function index(mixed $key): InsertIndexBuilder
    {
        return new InsertIndexBuilder(to_db_value($key));
    }

    public function nodes(): InsertNodesBuilder
    {
        return new InsertNodesBuilder();
    }

    public function values(array $data): InsertValuesBuilder
    {
        return new InsertValuesBuilder(to_multi_values($data));
    }

    public function values_uniform(array $data): InsertValuesBuilder
    {
        return new InsertValuesBuilder(to_single_values($data));
    }
}

class RemoveAliasesBuilder
{
    private array $data;

    public function __construct(array $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['remove_aliases' => $this->data]);
    }
}

class RemoveIdsBuilder
{
    private QueryIds $data;

    public function __construct(QueryIds $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['remove' => $this->data]);
    }
}

class RemoveIndexBuilder
{
    private DbValue $data;

    public function __construct(DbValue $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['remove_index' => $this->data]);
    }
}

class RemoveValuesBuilder
{
    private SelectValuesQuery $data;

    public function __construct(SelectValuesQuery $data)
    {
        $this->data = $data;
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): RemoveValuesBuilder
    {
        $this->data->setIds(to_query_ids($ids));
        return $this;
    }

    public function query(): QueryType
    {
        return new QueryType(['remove_values' => $this->data]);
    }
}

class RemoveBuilder
{
    public function aliases(string|array $names): RemoveAliasesBuilder
    {
        return new RemoveAliasesBuilder(is_array($names) ? $names : array($names));
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): RemoveIdsBuilder
    {
        return new RemoveIdsBuilder(to_query_ids($ids));
    }

    public function index(int|float|string|DbValue $value): RemoveIndexBuilder
    {
        return new RemoveIndexBuilder(to_db_value($value));
    }

    public function values(array $data): RemoveValuesBuilder
    {
        return new RemoveValuesBuilder(new SelectValuesQuery(['keys' => to_db_keys($data)]));
    }
}

class SearchAlgorithmBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function from(string|int|QueryId $id): SearchFromBuilder
    {
        $this->data->setOrigin(to_query_id($id));
        return new SearchFromBuilder($this->data);
    }

    public function to(string|int|QueryId $id): SearchToBuilder
    {
        $this->data->setDestination(to_query_id($id));
        return new SearchToBuilder($this->data);
    }
}

class SearchIndexValueBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['search' => $this->data]);
    }
}

class SearchIndexBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function value(int|float|string|array|DbValue $v): SearchIndexValueBuilder
    {
        $this->data->getConditions()[0]->getData()->getKeyValue()->setValue(new Comparison(['equal' => to_db_value($v)]));
        return new SearchIndexValueBuilder($this->data);
    }
}

class SearchOrderByBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function offset(int $offset): SearchOffsetBuilder
    {
        $this->data->setOffset($offset);
        return new SearchOffsetBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['search' => $this->data]);
    }
}

class SearchWhereKeyBuilder
{
    private SearchWhereBuilder $data;
    private DbValue $key;

    public function __construct(SearchWhereBuilder $data, DbValue $key)
    {
        $this->data = $data;
        $this->key = $key;
    }

    public function value(Comparison $comparison): SearchWhereLogicBuilder
    {
        $this->data->push_condition(new QueryConditionData(["key_value" => new QueryConditionDataOneOf5KeyValue(['key' => $this->key, 'comparison' => $comparison])]));
        return new SearchWhereLogicBuilder($this->data);
    }
}

class SearchWhereLogicBuilder
{
    private SearchWhereBuilder $data;

    public function __construct(SearchWhereBuilder $data)
    {
        $this->data = $data;
    }

    public function and(): SearchWhereBuilder
    {
        $this->data->logic = QueryConditionLogic::_AND;
        return $this->data;
    }

    public function end_where(): SearchWhereLogicBuilder
    {
        $this->data->collapse_conditions();
        return $this;
    }

    public function or(): SearchWhereBuilder
    {
        $this->data->logic = QueryConditionLogic::_OR;
        return $this->data;
    }

    public function query(): QueryType
    {
        while ($this->data->collapse_conditions()) {
        }
        return new QueryType(['search' => $this->data->data]);
    }
}

class SearchWhereBuilder
{
    private SearchQuery $data;
    private string $modifier = QueryConditionModifier::NONE;
    private string $logic = QueryConditionLogic::_AND;
    private array $conditions = [[]];

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function beyond(): SearchWhereBuilder
    {
        $this->modifier = QueryConditionModifier::BEYOND;
        return $this;
    }

    public function distance(CountComparison $comparison): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(["distance" => $comparison]));
        return new SearchWhereLogicBuilder($this);
    }

    public function edge(): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(['edge' => new stdClass()]));
        return new SearchWhereLogicBuilder($this);
    }

    public function edge_count(CountComparison $comparison): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(["edge_count" => $comparison]));
        return new SearchWhereLogicBuilder($this);
    }

    public function edge_count_from(CountComparison $comparison): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(["edge_count_from" => $comparison]));
        return new SearchWhereLogicBuilder($this);
    }

    public function edge_count_to(CountComparison $comparison): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(["edge_count_to" => $comparison]));
        return new SearchWhereLogicBuilder($this);
    }

    public function ids(string|int|array|QueryId|QueryIds $ids): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(["ids" => to_query_ids($ids)]));
        return new SearchWhereLogicBuilder($this);
    }

    public function key(int|float|string|array|DbValue $key): SearchWhereKeyBuilder
    {
        return new SearchWhereKeyBuilder($this, to_db_value($key));
    }

    public function keys(int|float|string|array|DbValue $keys): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(["keys" => is_array($keys) ? to_db_keys($keys) : array(to_db_value($keys))]));
        return new SearchWhereLogicBuilder($this);
    }

    public function node(): SearchWhereLogicBuilder
    {
        $this->push_condition(new QueryConditionData(['node' => new stdClass()]));
        return new SearchWhereLogicBuilder($this);
    }

    public function not(): SearchWhereBuilder
    {
        $this->modifier = QueryConditionModifier::NOT;
        return $this;
    }

    public function not_beyond(): SearchWhereBuilder
    {
        $this->modifier = QueryConditionModifier::NOT_BEYOND;
        return $this;
    }

    public function where(): SearchWhereBuilder
    {
        $this->push_condition(new QueryConditionData(['where' => []]));
        $this->conditions[] = [];
        return $this;
    }

    private function push_condition($data)
    {
        end($this->conditions)[] = new QueryCondition(['data' => $data, 'modifier' => $this->modifier, 'logic' => $this->logic]);
        $this->modifier = QueryConditionModifier::NONE;
        $this->logic = QueryConditionLogic::_AND;
    }

    private function collapse_conditions(): bool
    {
        $len = count($this->conditions);

        if ($len > 1) {
            $last = array_pop($this->conditions);
            $current = end($this->conditions);
            $last_condition = end($current);
            $last_condition->setData(new QueryConditionData(['where' => $last]));
            return true;
        }

        return false;
    }
}

class SearchFromBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function offset(int $offset): SearchOffsetBuilder
    {
        $this->data->setOffset($offset);
        return new SearchOffsetBuilder($this->data);
    }

    public function order_by(array $data): SearchOrderByBuilder
    {
        $this->data->setOrderBy(is_array($data) ? $data : array($data));
        return new SearchOrderByBuilder($this->data);
    }

    public function to(string|int|QueryId $id): SearchToBuilder
    {
        $this->data->setDestination(to_query_id($id));
        return new SearchToBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['search' => $this->data]);
    }
}

class SearchLimitBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['search' => $this->data]);
    }
}

class SearchOffsetBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['search' => $this->data]);
    }
}

class SearchToBuilder
{
    private SearchQuery $data;

    public function __construct(SearchQuery $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function offset(int $offset): SearchOffsetBuilder
    {
        $this->data->setOffset($offset);
        return new SearchOffsetBuilder($this->data);
    }

    public function order_by(array $data): SearchOrderByBuilder
    {
        $this->data->setOrderBy(is_array($data) ? $data : array($data));
        return new SearchOrderByBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(['search' => $this->data]);
    }
}

class SearchBuilder
{
    public function breadth_first(): SearchAlgorithmBuilder
    {
        return new SearchAlgorithmBuilder(new SearchQuery(['algorithm' => SearchQueryAlgorithm::BREADTH_FIRST]));
    }

    public function depth_first(): SearchAlgorithmBuilder
    {
        return new SearchAlgorithmBuilder(new SearchQuery(['algorithm' => SearchQueryAlgorithm::DEPTH_FIRST]));
    }

    public function elements(): SearchToBuilder
    {
        return new SearchToBuilder(new SearchQuery(['algorithm' => SearchQueryAlgorithm::ELEMENTS]));
    }

    public function from(string|int|QueryId $id): SearchFromBuilder
    {
        return new SearchFromBuilder(new SearchQuery(['origin' => to_query_id($id)]));
    }

    public function index(int|float|string|array|DbValue $key): SearchIndexBuilder
    {
        $condition = new QueryCondition(['data' => new QueryConditionData(['key_value' => new QueryConditionDataOneOf5KeyValue(['key' => to_db_value($key)])])]);
        return new SearchIndexBuilder(new SearchQuery(['algorithm' => SearchQueryAlgorithm::INDEX, 'conditions' => [$condition]]));
    }

    public function to(string|int|QueryId $id): SearchToBuilder
    {
        return new SearchToBuilder(new SearchQuery(['destination' => to_query_id($id)]));
    }
}

class SelectAliasesIdsBuilder
{
    private QueryIds $data;

    public function __construct(QueryIds $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['select_aliases' => $this->data]);
    }
}

class SelectAliasesBuilder
{
    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): SelectAliasesIdsBuilder
    {
        return new SelectAliasesIdsBuilder(to_query_ids($ids));
    }

    public function query(): QueryType
    {
        return new QueryType(['select_all_aliases' => new stdClass()]);
    }
}

class SelectEdgeCountIdsBuilder
{
    private SelectEdgeCountQuery $data;

    public function __construct(SelectEdgeCountQuery $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['select_edge_count' => $this->data]);
    }
}

class SelectEdgeCountBuilder
{
    private bool $from;
    private bool $to;

    public function __construct(bool $from, bool $to)
    {
        $this->from = $from;
        $this->to = $to;
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): SelectEdgeCountIdsBuilder
    {
        return new SelectEdgeCountIdsBuilder(new SelectEdgeCountQuery(['from' => $this->from, 'to' => $this->to, 'ids' => to_query_ids($ids)]));
    }
}

class SelectValuesIdsBuilder
{
    private SelectValuesQuery $data;

    public function __construct(SelectValuesQuery $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['select_values' => $this->data]);
    }
}

class SelectValuesBuilder
{
    private SelectValuesQuery $data;

    public function __construct(SelectValuesQuery $data)
    {
        $this->data = $data;
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): SelectValuesIdsBuilder
    {
        $this->data->setIds(to_query_ids($ids));
        return new SelectValuesIdsBuilder($this->data);
    }
}

class SelectIndexesBuilder
{
    public function query(): QueryType
    {
        return new QueryType(['select_all_indexes' => new stdClass()]);
    }
}

class SelectKeysIdsBuilder
{
    private QueryIds $data;

    public function __construct(QueryIds $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['select_keys' => $this->data]);
    }
}

class SelectKeysBuilder
{
    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): SelectKeysIdsBuilder
    {
        return new SelectKeysIdsBuilder(to_query_ids($ids));
    }
}

class SelectKeyCountIdsBuilder
{
    private QueryIds $data;

    public function __construct(QueryIds $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return new QueryType(['select_key_count' => $this->data]);
    }
}

class SelectKeyCountBuilder
{
    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): SelectKeyCountIdsBuilder
    {
        return new SelectKeyCountIdsBuilder(to_query_ids($ids));
    }
}

class SelectNodeCountBuilder
{
    public function query(): QueryType
    {
        return new QueryType(['select_node_count' => new stdClass()]);
    }
}

class SelectBuilder
{
    public function aliases(): SelectAliasesBuilder
    {
        return new SelectAliasesBuilder();
    }

    public function edge_count_to(): SelectEdgeCountBuilder
    {
        return new SelectEdgeCountBuilder(false, true);
    }

    public function edge_count_from(): SelectEdgeCountBuilder
    {
        return new SelectEdgeCountBuilder(true, false);
    }

    public function edge_count(): SelectEdgeCountBuilder
    {
        return new SelectEdgeCountBuilder(true, true);
    }

    public function ids(string|int|array|QueryId|SearchQuery|QueryIds $ids): SelectValuesIdsBuilder
    {
        return new SelectValuesIdsBuilder(new SelectValuesQuery(['ids' => to_query_ids($ids), 'keys' => []]));
    }

    public function indexes(): SelectIndexesBuilder
    {
        return new SelectIndexesBuilder();
    }

    public function keys(): SelectKeysBuilder
    {
        return new SelectKeysBuilder();
    }

    public function key_count(): SelectKeyCountBuilder
    {
        return new SelectKeyCountBuilder();
    }

    public function node_count(): SelectNodeCountBuilder
    {
        return new SelectNodeCountBuilder();
    }

    public function values(array $data): SelectValuesBuilder
    {
        return new SelectValuesBuilder(new SelectValuesQuery(['ids' => [], 'keys' => to_db_keys($data)]));
    }
}

class QueryBuilder
{
    public static function insert(): InsertBuilder
    {
        return new InsertBuilder();
    }

    public static function remove(): RemoveBuilder
    {
        return new RemoveBuilder();
    }

    public static function search(): SearchBuilder
    {
        return new SearchBuilder();
    }

    public static function select(): SelectBuilder
    {
        return new SelectBuilder();
    }
}
