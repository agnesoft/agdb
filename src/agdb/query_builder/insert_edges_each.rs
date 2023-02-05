use super::insert_edges_values::InsertEdgesValues;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::query::Query;
use crate::DbKeyValue;

pub struct InsertEdgesEach(pub InsertEdgesQuery);

impl InsertEdgesEach {
    pub fn query(self) -> Query {
        Query::InsertEdges(self.0)
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertEdgesValues(self.0)
    }

    pub fn values_id(mut self, id: QueryId) -> InsertEdgesValues {
        self.0.values = QueryValues::Ids(QueryIds::Id(id));

        InsertEdgesValues(self.0)
    }

    pub fn values_ids(mut self, ids: &[QueryId]) -> InsertEdgesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(ids.to_vec()));

        InsertEdgesValues(self.0)
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertEdgesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertEdgesValues(self.0)
    }
}
