<?php

namespace Agnesoft\AgdbApi;

use Agnesoft\AgdbApi\Model\KeyValueComparison;
use stdClass;
use Agnesoft\AgdbApi\Model\CountComparison;
use Agnesoft\AgdbApi\Model\QueryConditionLogic;
use Agnesoft\AgdbApi\Model\QueryConditionModifier;
use Agnesoft\AgdbApi\Model\Comparison;
use Agnesoft\AgdbApi\Model\QueryCondition;
use Agnesoft\AgdbApi\Model\QueryConditionData;
use Agnesoft\AgdbApi\Model\DbKeyValue;
use Agnesoft\AgdbApi\Model\DbValue;
use Agnesoft\AgdbApi\Model\InsertAliasesQuery;
use Agnesoft\AgdbApi\Model\InsertEdgesQuery;
use Agnesoft\AgdbApi\Model\InsertNodesQuery;
use Agnesoft\AgdbApi\Model\InsertValuesQuery;
use Agnesoft\AgdbApi\Model\QueryId;
use Agnesoft\AgdbApi\Model\QueryIds;
use Agnesoft\AgdbApi\Model\QueryType;
use Agnesoft\AgdbApi\Model\QueryValues;
use Agnesoft\AgdbApi\Model\SearchQuery;
use Agnesoft\AgdbApi\Model\SelectValuesQuery;
use Agnesoft\AgdbApi\Model\SelectEdgeCountQuery;
use Agnesoft\AgdbApi\Model\SearchQueryAlgorithm;
use Agnesoft\AgdbApi\Model\DbKeyOrder;

class SearchQueryBuilder
{
    public QueryType|null $query;
    public SearchQuery $search;

    public function __construct(QueryType|null $query)
    {
        $search = self::new_search();
        $this->query = $query;

        if ($query) {
            // @phpstan-ignore-next-line if.alwaysTrue
            if ($query->getInsertValues()) {
                $query
                    ->getInsertValues()
                    ->setIds(new QueryIds(["search" => $search]));
                // @phpstan-ignore-next-line if.alwaysTrue
            } elseif ($query->getRemove()) {
                $query->getRemove()->setSearch($search);
                // @phpstan-ignore-next-line if.alwaysTrue
            } elseif ($query->getRemoveValues()) {
                $query
                    ->getRemoveValues()
                    ->setIds(new QueryIds(["search" => $search]));
                // @phpstan-ignore-next-line if.alwaysTrue
            } elseif ($query->getSelectAliases()) {
                $query->getSelectAliases()->setSearch($search);
                // @phpstan-ignore-next-line if.alwaysTrue
            } elseif ($query->getSelectEdgeCount()) {
                $query
                    ->getSelectEdgeCount()
                    ->setIds(new QueryIds(["search" => $search]));
                // @phpstan-ignore-next-line if.alwaysTrue
            } elseif ($query->getSelectKeys()) {
                $query->getSelectKeys()->setSearch($search);
                // @phpstan-ignore-next-line if.alwaysTrue
            } elseif ($query->getSelectKeyCount()) {
                $query->getSelectKeyCount()->setSearch($search);
            } elseif ($query->getSelectValues()) {
                $query
                    ->getSelectValues()
                    ->setIds(new QueryIds(["search" => $search]));
            }
        }

        $this->search = $search;
    }

    private static function new_search(): SearchQuery
    {
        $query = new SearchQuery();
        $query->setAlgorithm(SearchQueryAlgorithm::BREADTH_FIRST); // @phpstan-ignore argument.type
        $query->setOrigin(new QueryId(["id" => 0]));
        $query->setDestination(new QueryId(["id" => 0]));
        $query->setOffset(0);
        $query->setLimit(0);
        $query->setOrderBy([]);
        $query->setConditions([]);
        return $query;
    }
}

class CountComparisonBuilder
{
    public static function Equal(int $value): CountComparison
    {
        return new CountComparison(["equal" => $value]);
    }

    public static function GreaterThan(int $value): CountComparison
    {
        return new CountComparison(["greater_than" => $value]);
    }

    public static function GreaterThanOrEqual(int $value): CountComparison
    {
        return new CountComparison(["greater_than_or_equal" => $value]);
    }

    public static function LessThan(int $value): CountComparison
    {
        return new CountComparison(["less_than" => $value]);
    }

    public static function LessThanOrEqual(int $value): CountComparison
    {
        return new CountComparison(["less_than_or_equal" => $value]);
    }

    public static function NotEqual(int $value): CountComparison
    {
        return new CountComparison(["not_equal" => $value]);
    }
}

class ComparisonBuilder
{
    public static function Contains(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["contains" => to_db_value($value)]);
    }

    public static function Equal(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["equal" => to_db_value($value)]);
    }

    public static function GreaterThan(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["greater_than" => to_db_value($value)]);
    }

    public static function GreaterThanOrEqual(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["greater_than_or_equal" => to_db_value($value)]);
    }

    public static function LessThan(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["less_than" => to_db_value($value)]);
    }

    public static function LessThanOrEqual(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["less_than_or_equal" => to_db_value($value)]);
    }

    public static function NotEqual(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["not_equal" => to_db_value($value)]);
    }

    public static function StartsWith(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["starts_with" => to_db_value($value)]);
    }

    public static function EndsWith(
        bool|int|float|string|array|DbValue $value
    ): Comparison {
        return new Comparison(["ends_with" => to_db_value($value)]);
    }
}

class DbKeyOrderBuilder
{
    public static function Asc(
        int|float|string|array|DbValue $value
    ): DbKeyOrder {
        return new DbKeyOrder(["asc" => to_db_value($value)]);
    }

    public static function Desc(
        int|float|string|array|DbValue $value
    ): DbKeyOrder {
        return new DbKeyOrder(["desc" => to_db_value($value)]);
    }
}

function to_query_id(string|int|QueryId $id): QueryId
{
    if (is_string($id)) {
        return new QueryId(["alias" => $id]);
    }

    if (is_int($id)) {
        return new QueryId(["id" => $id]);
    }

    return $id;
}

function to_query_ids(
    string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
): QueryIds {
    $query_ids = new QueryIds();

    if (is_string($ids)) {
        $id = new QueryId();
        $id->setAlias($ids);
        $query_ids->setIds([$id]);
    } elseif (is_int($ids)) {
        $id = new QueryId();
        $id->setId($ids);
        $query_ids->setIds([$id]);
    } elseif (is_array($ids)) {
        if (count($ids) !== 0) {
            if (is_int($ids[0])) {
                return new QueryIds([
                    "ids" => array_map(
                        fn($id) => new QueryId(["id" => $id]),
                        $ids
                    ),
                ]);
            } else {
                return new QueryIds([
                    "ids" => array_map(
                        fn($id) => new QueryId(["alias" => $id]),
                        $ids
                    ),
                ]);
            }
        } else {
            $query_ids->setIds([]);
        }
    } elseif (get_class($ids) === QueryId::class) {
        $query_ids->setIds([$ids]);
    } elseif (get_class($ids) === QueryType::class) {
        $query_ids->setSearch($ids->getSearch());
    } elseif (get_class($ids) == SearchQuery::class) {
        $query_ids->setSearch($ids);
    }

    return $query_ids;
}

function to_db_value(bool|int|float|string|array|DbValue $value): DbValue
{
    if (is_string($value)) {
        return new DbValue(["string" => $value]);
    }

    if (is_int($value)) {
        return new DbValue(["i64" => $value]);
    }

    if (is_float($value)) {
        return new DbValue(["f64" => $value]);
    }

    if (is_bool($value)) {
        return new DbValue(["string" => $value ? "true" : "false"]);
    }

    if (is_array($value)) {
        if (count($value) === 0) {
            return new DbValue(["vec_i64" => []]);
        }

        if (is_int($value[0])) {
            return new DbValue(["vec_i64" => $value]);
        }

        if (is_float($value[0])) {
            return new DbValue(["vec_f64" => $value]);
        }

        return new DbValue(["vec_string" => $value]);
    }

    return $value;
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
            $element_values[] = new DbKeyValue([
                "key" => to_db_value($key),
                "value" => to_db_value($value),
            ]);
        }

        $values[] = $element_values;
    }

    return new QueryValues(["multi" => $values]);
}

function to_single_values(array $data): QueryValues
{
    $values = [];

    foreach ($data as $key => $value) {
        $values[] = new DbKeyValue([
            "key" => to_db_value($key),
            "value" => to_db_value($value),
        ]);
    }

    return new QueryValues(["single" => $values]);
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
        return new QueryType(["insert_alias" => $this->data]);
    }
}

class InsertAliasesBuilder
{
    private InsertAliasesQuery $data;

    function __construct(array $names)
    {
        $this->data = new InsertAliasesQuery();
        $this->data->setAliases($names);
    }

    function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertAliasesIdsBuilder {
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
        return new QueryType(["insert_values" => $this->data]);
    }
}

class InsertEdgesToEachBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function values(array $values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function values_uniform(array $values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(["insert_edges" => $this->data]);
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
        return new QueryType(["insert_edges" => $this->data]);
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

    public function values(array $values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function values_uniform(array $values): InsertEdgesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertEdgesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(["insert_edges" => $this->data]);
    }
}

class InsertEdgesFromBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function to(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertEdgesToBuilder {
        $this->data->setTo(to_query_ids($ids));
        return new InsertEdgesToBuilder($this->data);
    }
}

class InsertEdgesIdsBuilder
{
    private InsertEdgesQuery $data;

    public function __construct(InsertEdgesQuery $data)
    {
        $this->data = $data;
    }

    public function from(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertEdgesFromBuilder {
        $this->data->setFrom(to_query_ids($ids));
        return new InsertEdgesFromBuilder($this->data);
    }
}

class InsertEdgesBuilder
{
    private InsertEdgesQuery $data;

    public function __construct()
    {
        $this->data = self::new_query();
    }

    public function from(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertEdgesFromBuilder {
        $this->data->setFrom(to_query_ids($ids));
        return new InsertEdgesFromBuilder($this->data);
    }

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertEdgesIdsBuilder {
        $this->data->setIds(to_query_ids($ids));
        return new InsertEdgesIdsBuilder($this->data);
    }

    private static function new_query(): InsertEdgesQuery
    {
        $query = new InsertEdgesQuery();
        $ids = new QueryIds();
        $ids->setIds([]);
        $query->setIds($ids);
        $values = new QueryValues();
        $values->setSingle([]);
        $query->setValues($values);
        $query->setEach(false);
        return $query;
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
        return new QueryType(["insert_nodes" => $this->data]);
    }
}

class InsertNodesAliasesBuilder
{
    private InsertNodesQuery $data;

    public function __construct(InsertNodesQuery $data)
    {
        $this->data = $data;
    }

    public function values(array $values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function values_uniform(array $values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(["insert_nodes" => $this->data]);
    }
}

class InsertNodesCountBuilder
{
    private InsertNodesQuery $data;

    public function __construct(InsertNodesQuery $data)
    {
        $this->data = $data;
    }

    public function values_uniform(array $values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function query(): QueryType
    {
        return new QueryType(["insert_nodes" => $this->data]);
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
        $this->data->setAliases(is_array($names) ? $names : [$names]);
        return new InsertNodesAliasesBuilder($this->data);
    }

    public function count(int $count): InsertNodesCountBuilder
    {
        $this->data->setCount($count);
        return new InsertNodesCountBuilder($this->data);
    }

    public function values(array $values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }

    public function values_uniform(array $values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_single_values($values));
        return new InsertNodesValuesBuilder($this->data);
    }
}

class InsertNodesBuilder
{
    private InsertNodesQuery $data;

    public function __construct()
    {
        $this->data = new InsertNodesQuery();
        $this->data->setAliases([]);
        $this->data->setCount(0);
        $ids = new QueryIds();
        $ids->setIds([]);
        $this->data->setIds($ids);
        $values = new QueryValues();
        $values->setSingle([]);
        $this->data->setValues($values);
    }

    public function aliases(string|array $names): InsertNodesAliasesBuilder
    {
        $this->data->setAliases(is_array($names) ? $names : [$names]);
        return new InsertNodesAliasesBuilder($this->data);
    }

    public function count(int $count): InsertNodesCountBuilder
    {
        $this->data->setCount($count);
        return new InsertNodesCountBuilder($this->data);
    }

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertNodesIdsBuilder {
        $this->data->setIds(to_query_ids($ids));
        return new InsertNodesIdsBuilder($this->data);
    }

    public function values(array $values): InsertNodesValuesBuilder
    {
        $this->data->setValues(to_multi_values($values));
        return new InsertNodesValuesBuilder($this->data);
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
        return new QueryType(["insert_index" => $this->data]);
    }
}

class InsertValuesBuilder
{
    private QueryValues $data;

    public function __construct(QueryValues $data)
    {
        $this->data = $data;
    }

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): InsertValuesIdsBuilder {
        return new InsertValuesIdsBuilder(
            new InsertValuesQuery([
                "values" => $this->data,
                "ids" => to_query_ids($ids),
            ])
        );
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType([
                "insert_values" => new InsertValuesQuery([
                    "values" => $this->data,
                    "ids" => new QueryIds(),
                ]),
            ])
        );
    }
}

class InsertBuilder
{
    public function aliases(string|array $names): InsertAliasesBuilder
    {
        return new InsertAliasesBuilder(is_array($names) ? $names : [$names]);
    }

    public function element(mixed $elem): InsertValuesIdsBuilder
    {
        return $this->elements([$elem]);
    }

    public function elements(array $elems): InsertValuesIdsBuilder
    {
        $data = new InsertValuesQuery();
        $values = [];
        $ids = [];

        foreach ($elems as $elem) {
            $element_values = [];

            if (!property_exists($elem, "db_id")) {
                $ids[] = new QueryId(["id" => 0]);
            }

            foreach ($elem as $key => $value) {
                if ($key === "db_id") {
                    if (is_null($value) || $value === 0) {
                        $ids[] = new QueryId(["id" => 0]);
                    } else {
                        $ids[] = to_query_id($value);
                    }
                } else {
                    $element_values[] = new DbKeyValue([
                        "key" => new DbValue(["string" => $key]),
                        "value" => to_db_value($value),
                    ]);
                }
            }

            $values[] = $element_values;
        }

        $data->setValues(new QueryValues(["multi" => $values]));
        $data->setIds(new QueryIds(["ids" => $ids]));

        return new InsertValuesIdsBuilder($data);
    }

    public function edges(): InsertEdgesBuilder
    {
        return new InsertEdgesBuilder();
    }

    public function index(
        int|float|string|array|DbValue $key
    ): InsertIndexBuilder {
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
        return new QueryType(["remove_aliases" => $this->data]);
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
        return new QueryType(["remove" => $this->data]);
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
        return new QueryType(["remove_index" => $this->data]);
    }
}

class RemoveValuesBuilder
{
    private SelectValuesQuery $data;

    public function __construct(SelectValuesQuery $data)
    {
        $this->data = $data;
    }

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): RemoveValuesBuilder {
        $this->data->setIds(to_query_ids($ids));
        return $this;
    }

    public function query(): QueryType
    {
        return new QueryType(["remove_values" => $this->data]);
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType(["remove_values" => $this->data])
        );
    }
}

class RemoveBuilder
{
    public function aliases(string|array $names): RemoveAliasesBuilder
    {
        return new RemoveAliasesBuilder(is_array($names) ? $names : [$names]);
    }

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): RemoveIdsBuilder {
        return new RemoveIdsBuilder(to_query_ids($ids));
    }

    public function index(int|float|string|DbValue $value): RemoveIndexBuilder
    {
        return new RemoveIndexBuilder(to_db_value($value));
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(new QueryType(["remove" => new QueryIds()]));
    }

    public function values(array $data): RemoveValuesBuilder
    {
        return new RemoveValuesBuilder(
            new SelectValuesQuery(["keys" => to_db_keys($data)])
        );
    }
}

class SearchAlgorithmBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function from(string|int|QueryId $id): SearchFromBuilder
    {
        $this->data->search->setOrigin(to_query_id($id));
        return new SearchFromBuilder($this->data);
    }

    public function to(string|int|QueryId $id): SearchToBuilder
    {
        $this->data->search->setDestination(to_query_id($id));
        return new SearchToBuilder($this->data);
    }
}

class SearchIndexValueBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function query(): QueryType
    {
        return $this->data->query != null
            ? $this->data->query
            : new QueryType(["search" => $this->data->search]);
    }
}

class SearchIndexBuilder
{
    private SearchQueryBuilder $data;
    private DbValue $key;

    public function __construct(SearchQueryBuilder $data, DbValue $key)
    {
        $this->data = $data;
        $this->key = $key;
    }

    public function value(
        int|float|string|array|DbValue $v
    ): SearchIndexValueBuilder {
        $condition = new QueryCondition();
        $condition->setLogic(QueryConditionLogic::_AND); // @phpstan-ignore argument.type
        $condition->setModifier(QueryConditionModifier::NONE); // @phpstan-ignore argument.type
        $condition_data = new QueryConditionData();
        $kvc = new KeyValueComparison();
        $kvc->setKey($this->key);
        $kvc->setValue(new Comparison(["equal" => to_db_value($v)]));
        $condition_data->setKeyValue($kvc);
        $condition->setData($condition_data);
        $this->data->search->setConditions([$condition]);
        return new SearchIndexValueBuilder($this->data);
    }
}

class SearchOrderByBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->search->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function offset(int $offset): SearchOffsetBuilder
    {
        $this->data->search->setOffset($offset);
        return new SearchOffsetBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return $this->data->query != null
            ? $this->data->query
            : new QueryType(["search" => $this->data->search]);
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

    public function value(
        Comparison|bool|int|float|string|array|DbValue $comparison
    ): SearchWhereLogicBuilder {
        if (!($comparison instanceof Comparison)) {
            $comparison = ComparisonBuilder::Equal(to_db_value($comparison));
        }

        $condition_data = new QueryConditionData();
        $kvc = new KeyValueComparison();
        $kvc->setKey($this->key);
        $kvc->setValue($comparison);
        $condition_data->setKeyValue($kvc);
        $this->data->__push_condition($condition_data);
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
        $this->data->__logic = QueryConditionLogic::_AND;
        return $this->data;
    }

    public function end_where(): SearchWhereLogicBuilder
    {
        $this->data->__collapse_conditions();
        return $this;
    }

    public function or(): SearchWhereBuilder
    {
        $this->data->__logic = QueryConditionLogic::_OR;
        return $this->data;
    }

    public function query(): QueryType
    {
        while ($this->data->__collapse_conditions()) {
        }
        $this->data->__data->search->setConditions(
            $this->data->__conditions[0]
        );
        return $this->data->__data->query ?:
            new QueryType(["search" => $this->data->__data->search]);
    }
}

class SearchWhereBuilder
{
    public SearchQueryBuilder $__data;
    public string $__modifier = QueryConditionModifier::NONE;
    public string $__logic = QueryConditionLogic::_AND;
    public array $__conditions = [[]];

    public function __construct(SearchQueryBuilder $data)
    {
        $this->__data = $data;
    }

    public function beyond(): SearchWhereBuilder
    {
        $this->__modifier = QueryConditionModifier::BEYOND;
        return $this;
    }

    public function distance(
        CountComparison|int $comparison
    ): SearchWhereLogicBuilder {
        if (is_int($comparison)) {
            $comparison = CountComparisonBuilder::Equal($comparison);
        }
        $this->__push_condition(
            new QueryConditionData(["distance" => $comparison])
        );
        return new SearchWhereLogicBuilder($this);
    }

    public function edge(): SearchWhereLogicBuilder
    {
        $this->__push_condition("Edge");
        return new SearchWhereLogicBuilder($this);
    }

    public function edge_count(
        CountComparison|int $comparison
    ): SearchWhereLogicBuilder {
        if (is_int($comparison)) {
            $comparison = CountComparisonBuilder::Equal($comparison);
        }
        $this->__push_condition(
            new QueryConditionData(["edge_count" => $comparison])
        );
        return new SearchWhereLogicBuilder($this);
    }

    public function edge_count_from(
        CountComparison|int $comparison
    ): SearchWhereLogicBuilder {
        if (is_int($comparison)) {
            $comparison = CountComparisonBuilder::Equal($comparison);
        }
        $this->__push_condition(
            new QueryConditionData(["edge_count_from" => $comparison])
        );
        return new SearchWhereLogicBuilder($this);
    }

    public function edge_count_to(
        CountComparison|int $comparison
    ): SearchWhereLogicBuilder {
        if (is_int($comparison)) {
            $comparison = CountComparisonBuilder::Equal($comparison);
        }
        $this->__push_condition(
            new QueryConditionData(["edge_count_to" => $comparison])
        );
        return new SearchWhereLogicBuilder($this);
    }

    public function ids(
        string|int|array|QueryId|QueryIds $ids
    ): SearchWhereLogicBuilder {
        $this->__push_condition(
            new QueryConditionData(["ids" => to_query_ids($ids)->getIds()])
        );
        return new SearchWhereLogicBuilder($this);
    }

    public function key(
        int|float|string|array|DbValue $key
    ): SearchWhereKeyBuilder {
        return new SearchWhereKeyBuilder($this, to_db_value($key));
    }

    public function keys(
        int|float|string|array|DbValue $keys
    ): SearchWhereLogicBuilder {
        $this->__push_condition(
            new QueryConditionData([
                "keys" => is_array($keys)
                    ? to_db_keys($keys)
                    : [to_db_value($keys)],
            ])
        );
        return new SearchWhereLogicBuilder($this);
    }

    public function node(): SearchWhereLogicBuilder
    {
        $this->__push_condition("Node");
        return new SearchWhereLogicBuilder($this);
    }

    public function not(): SearchWhereBuilder
    {
        $this->__modifier = QueryConditionModifier::NOT;
        return $this;
    }

    public function not_beyond(): SearchWhereBuilder
    {
        $this->__modifier = QueryConditionModifier::NOT_BEYOND;
        return $this;
    }

    public function where(): SearchWhereBuilder
    {
        $this->__push_condition(new QueryConditionData(["where" => []]));
        $this->__conditions[] = [];
        return $this;
    }

    public function __push_condition(string|QueryConditionData $data): void
    {
        $count = count($this->__conditions);
        array_push(
            $this->__conditions[$count - 1],
            new QueryCondition([
                "data" => $data,
                "modifier" => $this->__modifier,
                "logic" => $this->__logic,
            ])
        );
        $this->__modifier = QueryConditionModifier::NONE;
        $this->__logic = QueryConditionLogic::_AND;
    }

    public function __collapse_conditions(): bool
    {
        $len = count($this->__conditions);

        if ($len > 1) {
            $last = array_pop($this->__conditions);
            $current = end($this->__conditions);
            $last_condition = end($current);
            $last_condition->setData(
                new QueryConditionData(["where" => $last])
            );
            return true;
        }

        return false;
    }
}

class SearchFromBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->search->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function offset(int $offset): SearchOffsetBuilder
    {
        $this->data->search->setOffset($offset);
        return new SearchOffsetBuilder($this->data);
    }

    public function order_by(DbKeyOrder|array $data): SearchOrderByBuilder
    {
        $this->data->search->setOrderBy(is_array($data) ? $data : [$data]);
        return new SearchOrderByBuilder($this->data);
    }

    public function to(string|int|QueryId $id): SearchToBuilder
    {
        $this->data->search->setDestination(to_query_id($id));
        return new SearchToBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return $this->data->query != null
            ? $this->data->query
            : new QueryType(["search" => $this->data->search]);
    }
}

class SearchLimitBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return $this->data->query != null
            ? $this->data->query
            : new QueryType(["search" => $this->data->search]);
    }
}

class SearchOffsetBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->search->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return $this->data->query != null
            ? $this->data->query
            : new QueryType(["search" => $this->data->search]);
    }
}

class SearchToBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(SearchQueryBuilder $data)
    {
        $this->data = $data;
    }

    public function limit(int $limit): SearchLimitBuilder
    {
        $this->data->search->setLimit($limit);
        return new SearchLimitBuilder($this->data);
    }

    public function offset(int $offset): SearchOffsetBuilder
    {
        $this->data->search->setOffset($offset);
        return new SearchOffsetBuilder($this->data);
    }

    public function order_by(DbKeyOrder|array $data): SearchOrderByBuilder
    {
        $this->data->search->setOrderBy(is_array($data) ? $data : [$data]);
        return new SearchOrderByBuilder($this->data);
    }

    public function where(): SearchWhereBuilder
    {
        return new SearchWhereBuilder($this->data);
    }

    public function query(): QueryType
    {
        return $this->data->query != null
            ? $this->data->query
            : new QueryType(["search" => $this->data->search]);
    }
}

class SearchBuilder
{
    private SearchQueryBuilder $data;

    public function __construct(QueryType|null $query)
    {
        $this->data = new SearchQueryBuilder($query);
    }

    public function breadth_first(): SearchAlgorithmBuilder
    {
        $this->data->search->setAlgorithm(SearchQueryAlgorithm::BREADTH_FIRST); // @phpstan-ignore argument.type
        return new SearchAlgorithmBuilder($this->data);
    }

    public function depth_first(): SearchAlgorithmBuilder
    {
        $this->data->search->setAlgorithm(SearchQueryAlgorithm::DEPTH_FIRST); // @phpstan-ignore argument.type
        return new SearchAlgorithmBuilder($this->data);
    }

    public function elements(): SearchToBuilder
    {
        $this->data->search->setAlgorithm(SearchQueryAlgorithm::ELEMENTS); // @phpstan-ignore argument.type
        return new SearchToBuilder($this->data);
    }

    public function from(string|int|QueryId $id): SearchFromBuilder
    {
        $this->data->search->setOrigin(to_query_id($id));
        return new SearchFromBuilder($this->data);
    }

    public function index(
        int|float|string|array|DbValue $key
    ): SearchIndexBuilder {
        $this->data->search->setAlgorithm(SearchQueryAlgorithm::INDEX); // @phpstan-ignore argument.type
        return new SearchIndexBuilder($this->data, to_db_value($key));
    }

    public function to(string|int|QueryId $id): SearchToBuilder
    {
        $this->data->search->setDestination(to_query_id($id));
        return new SearchToBuilder($this->data);
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
        return new QueryType(["select_aliases" => $this->data]);
    }
}

class SelectAliasesBuilder
{
    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): SelectAliasesIdsBuilder {
        return new SelectAliasesIdsBuilder(to_query_ids($ids));
    }

    public function query(): QueryType
    {
        return new QueryType(["select_all_aliases" => new stdClass()]);
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType(["select_aliases" => new QueryIds()])
        );
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
        return new QueryType(["select_edge_count" => $this->data]);
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

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): SelectEdgeCountIdsBuilder {
        return new SelectEdgeCountIdsBuilder(
            new SelectEdgeCountQuery([
                "from" => $this->from,
                "to" => $this->to,
                "ids" => to_query_ids($ids),
            ])
        );
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType([
                "select_edge_count" => new SelectEdgeCountQuery([
                    "from" => $this->from,
                    "to" => $this->to,
                    "ids" => new QueryIds(),
                ]),
            ])
        );
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
        return new QueryType(["select_values" => $this->data]);
    }
}

class SelectValuesBuilder
{
    private SelectValuesQuery $data;

    public function __construct(SelectValuesQuery $data)
    {
        $this->data = $data;
    }

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): SelectValuesIdsBuilder {
        $this->data->setIds(to_query_ids($ids));
        return new SelectValuesIdsBuilder($this->data);
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType(["select_values" => $this->data])
        );
    }
}

class SelectIndexesBuilder
{
    public function query(): QueryType
    {
        return new QueryType(["select_indexes" => new stdClass()]);
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
        return new QueryType(["select_keys" => $this->data]);
    }
}

class SelectKeysBuilder
{
    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): SelectKeysIdsBuilder {
        return new SelectKeysIdsBuilder(to_query_ids($ids));
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType(["select_keys" => new QueryIds()])
        );
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
        return new QueryType(["select_key_count" => $this->data]);
    }
}

class SelectKeyCountBuilder
{
    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): SelectKeyCountIdsBuilder {
        return new SelectKeyCountIdsBuilder(to_query_ids($ids));
    }

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType(["select_key_count" => new QueryIds()])
        );
    }
}

class SelectNodeCountBuilder
{
    public function query(): QueryType
    {
        return new QueryType(["select_node_count" => new stdClass()]);
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

    public function ids(
        string|int|array|QueryId|SearchQuery|QueryType|QueryIds $ids
    ): SelectValuesIdsBuilder {
        return new SelectValuesIdsBuilder(
            new SelectValuesQuery(["ids" => to_query_ids($ids), "keys" => []])
        );
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

    public function search(): SearchBuilder
    {
        return new SearchBuilder(
            new QueryType([
                "select_values" => new SelectValuesQuery([
                    "keys" => [],
                    "ids" => new QueryIds(),
                ]),
            ])
        );
    }

    public function values(array $data): SelectValuesBuilder
    {
        return new SelectValuesBuilder(
            new SelectValuesQuery(["ids" => [], "keys" => to_db_keys($data)])
        );
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
        return new SearchBuilder(null);
    }

    public static function select(): SelectBuilder
    {
        return new SelectBuilder();
    }
}
