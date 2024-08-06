<?php

namespace Agnesoft\Agdb;

use Agdb\Model\DbKeyValue;
use Agdb\Model\DbValue;
use Agdb\Model\InsertAliasesQuery;
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
