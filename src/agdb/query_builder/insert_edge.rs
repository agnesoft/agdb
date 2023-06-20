use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;
use crate::query::search_query::SearchQuery;

pub struct InsertEdgesEach(pub InsertEdgesQuery);

pub struct InsertEdges(pub InsertEdgesQuery);

pub struct InsertEdgesFrom(pub InsertEdgesQuery);

pub struct InsertEdgesFromTo(pub InsertEdgesQuery);

pub struct InsertEdgesValues(pub InsertEdgesQuery);

impl InsertEdgesEach {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }

    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }
}

impl InsertEdges {
    pub fn from<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFrom {
        self.0.from = ids.into();

        InsertEdgesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_search(mut self, query: SearchQuery) -> InsertEdgesFrom {
        self.0.from = QueryIds::Search(query);

        InsertEdgesFrom(self.0)
    }
}

impl InsertEdgesFrom {
    pub fn to<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFromTo {
        self.0.to = ids.into();

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

    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }

    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }
}

impl InsertEdgesValues {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }
}
