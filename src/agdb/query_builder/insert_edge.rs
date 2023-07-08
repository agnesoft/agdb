use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;

/// Insert edges builder that lets you add `from`
/// (origin) nodes.
pub struct InsertEdges(pub InsertEdgesQuery);

/// Insert edges builder that lets you add values.
pub struct InsertEdgesEach(pub InsertEdgesQuery);

/// Insert edges builder that lets you add `to`
/// (destination) nodes.
pub struct InsertEdgesFrom(pub InsertEdgesQuery);

/// Insert edges builder that lets you add values
/// or set `each`.
pub struct InsertEdgesFromTo(pub InsertEdgesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct InsertEdgesValues(pub InsertEdgesQuery);

impl InsertEdges {
    /// An id or list of ids from where the edges should come from (origin).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().from(1).to(2);
    /// QueryBuilder::insert().edges().from(1).to(QueryBuilder::search().from(1).query());
    /// ```
    pub fn from<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFrom {
        self.0.from = ids.into();

        InsertEdgesFrom(self.0)
    }
}

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

impl InsertEdgesFrom {
    pub fn to<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFromTo {
        self.0.to = ids.into();

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
