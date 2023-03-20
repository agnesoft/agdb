use super::insert_edge_values::InsertEdgeValues;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;

pub struct InsertEdgeFromTo(pub InsertEdgesQuery);

impl InsertEdgeFromTo {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[DbKeyValue]) -> InsertEdgeValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertEdgeValues(self.0)
    }

    pub fn values_id(mut self, id: QueryId) -> InsertEdgeValues {
        self.0.values = QueryValues::Ids(QueryIds::Id(id));

        InsertEdgeValues(self.0)
    }
}
