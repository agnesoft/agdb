<?php

namespace Agnesoft\Agdb;

use Agdb\Model\DbKeyValue;
use Agdb\Model\DbValue;
use Agdb\Model\InsertAliasesQuery;
use Agdb\Model\InsertEdgesQuery;
use Agdb\Model\InsertValuesQuery;
use Agdb\Model\QueryId;
use Agdb\Model\QueryIds;
use Agdb\Model\QueryType;
use Agdb\Model\QueryValues;
use Agdb\Model\SearchQuery;

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

    throw new \InvalidArgumentException('Unknown $ids type', 1);
}

function to_db_value($value): DbValue
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

    throw new \InvalidArgumentException('Unknown $value type', 1);
}

function to_multi_values($data): QueryValues
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

function to_single_values($data): QueryValues
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

class InsertNodesBuilder
{

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

class RemoveBuilder
{

}

class SearchBuilder
{

}

class SelectBuilder
{

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
