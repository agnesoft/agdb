use crate::InsertEdgesQuery;
use crate::QueryIds;
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

/// Insert edges builder with ids allowing insert
/// or update semantics that lets you add `from`
/// (origin) nodes.
pub struct InsertEdgesIds(pub InsertEdgesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct InsertEdgesValues(pub InsertEdgesQuery);

impl InsertEdges {
    /// An id or list of ids or search query from where the edges should come from (origin).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().from(1).to(2);
    /// QueryBuilder::insert().edges().from(1).to([2, 3]);
    /// QueryBuilder::insert().edges().from(1).to(QueryBuilder::search().from(1).query());
    /// ```
    pub fn from<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFrom {
        self.0.from = ids.into();

        InsertEdgesFrom(self.0)
    }

    /// Optional ids of edges (can be search sub-query) to be
    /// inserted or updated. If the list is empty the nodes will be
    /// inserted. If the list is not empty all ids must exist in the
    /// database and will be updated instead:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().ids(1).from(1);
    /// QueryBuilder::insert().edges().ids(1).from([1, 2]);
    /// QueryBuilder::insert().edges().ids(1).from(QueryBuilder::search().from(1).query());
    /// ```
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesIds {
        self.0.ids = ids.into();

        InsertEdgesIds(self.0)
    }
}

impl InsertEdgesIds {
    /// An id or list of ids or search query from where the edges should come from (origin).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().from(1).to(2);
    /// QueryBuilder::insert().edges().from(1).to([2, 3]);
    /// QueryBuilder::insert().edges().from(1).to(QueryBuilder::search().from(1).query());
    /// ```
    pub fn from<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFrom {
        self.0.from = ids.into();

        InsertEdgesFrom(self.0)
    }
}

impl InsertEdgesEach {
    /// Returns the built `InsertEdgesQuery` object.
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    /// List of lists of `key_values` to be inserted into the edges. There must be exactly
    /// as many lists as the number of created edges.
    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }

    /// List of `key_values` to be inserted into all created edges.
    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }
}

impl InsertEdgesFrom {
    /// An id or list of ids or search query to where the edges should go (destination).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().from(1).to(2).query();
    /// QueryBuilder::insert().edges().from(1).to(2).each();
    /// QueryBuilder::insert().edges().from(1).to(2).values([[("k", 1).into()]]);
    /// QueryBuilder::insert().edges().from(1).to(2).values_uniform([("k", 1).into()]);
    /// ```
    pub fn to<T: Into<QueryIds>>(mut self, ids: T) -> InsertEdgesFromTo {
        self.0.to = ids.into();

        InsertEdgesFromTo(self.0)
    }
}

impl InsertEdgesFromTo {
    /// A modifier to create edges from each origin (from) to each destination (to)
    /// even if the number of origins and destinations is the same. This modifier is assumed
    /// and thus not needed if they are already asymmetric.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().from(1).to(2).each().query();
    /// QueryBuilder::insert().edges().from(1).to(2).each().values([[("k", 1).into()]]);
    /// QueryBuilder::insert().edges().from(1).to(2).each().values_uniform([("k", 1).into()]);
    /// ```
    pub fn each(mut self) -> InsertEdgesEach {
        self.0.each = true;

        InsertEdgesEach(self.0)
    }

    /// Returns the built `InsertEdgesQuery` object.
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }

    /// List of lists of `key_values` to be inserted into the edges. There must be exactly
    /// as many lists as the number of created edges.
    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }

    /// List of `key_values` to be inserted into all created edges.
    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertEdgesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertEdgesValues(self.0)
    }
}

impl InsertEdgesValues {
    /// Returns the built `InsertEdgesQuery` object.
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }
}
