use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::query::search_query::SearchQuery;
use crate::DbKeyValue;

pub struct InsertEdge(pub InsertEdgesQuery);

pub struct InsertEdgeFrom(pub InsertEdgesQuery);

pub struct InsertEdgeFromTo(pub InsertEdgesQuery);

pub struct InsertEdgeValues(pub InsertEdgesQuery);

pub struct InsertEdgesEach(pub InsertEdgesQuery);

pub struct InsertEdges(pub InsertEdgesQuery);

pub struct InsertEdgesFrom(pub InsertEdgesQuery);

pub struct InsertEdgesFromTo(pub InsertEdgesQuery);

pub struct InsertEdgesValues(pub InsertEdgesQuery);

impl InsertEdge {
    pub fn from<T: Into<QueryId>>(mut self, id: T) -> InsertEdgeFrom {
        self.0.from = QueryIds::Ids(vec![id.into()]);

        InsertEdgeFrom(self.0)
    }
}

impl InsertEdgeFrom {
    pub fn to<T: Into<QueryId>>(mut self, id: T) -> InsertEdgeFromTo {
        self.0.to = QueryIds::Ids(vec![id.into()]);

        InsertEdgeFromTo(self.0)
    }
}

impl InsertEdgeFromTo {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[DbKeyValue]) -> InsertEdgeValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertEdgeValues(self.0)
    }

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertEdgeValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

        InsertEdgeValues(self.0)
    }
}

impl InsertEdgeValues {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }
}

impl InsertEdgesEach {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertEdgesValues(self.0)
    }

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

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

impl InsertEdges {
    pub fn from(mut self, ids: &[QueryId]) -> InsertEdgesFrom {
        self.0.from = QueryIds::Ids(ids.to_vec());

        InsertEdgesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_search(mut self, query: SearchQuery) -> InsertEdgesFrom {
        self.0.from = QueryIds::Search(query);

        InsertEdgesFrom(self.0)
    }
}

impl InsertEdgesFrom {
    pub fn to(mut self, ids: &[QueryId]) -> InsertEdgesFromTo {
        self.0.to = QueryIds::Ids(ids.to_vec());

        InsertEdgesFromTo(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_search(mut self, query: SearchQuery) -> InsertEdgesFromTo {
        self.0.to = QueryIds::Search(query);

        InsertEdgesFromTo(self.0)
    }
}

impl InsertEdgesFromTo {
    pub fn each(mut self) -> InsertEdgesEach {
        self.0.each = true;

        InsertEdgesEach(self.0)
    }

    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertEdgesValues(self.0)
    }

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

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

impl InsertEdgesValues {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }
}
