use crate::query::query_aliases::QueryAliases;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;
use crate::InsertNodesQuery;
use crate::QueryIds;

/// Insert nodes builder to add aliases or count
/// or values.
pub struct InsertNodes(pub InsertNodesQuery);

/// Insert nodes builder to add values.
pub struct InsertNodesAliases(pub InsertNodesQuery);

/// Insert nodes builder to add uniform values.
pub struct InsertNodesCount(pub InsertNodesQuery);

/// Insert nodes builder to add aliases or count
/// or values.
pub struct InsertNodesIds(pub InsertNodesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct InsertNodesValues(pub InsertNodesQuery);

impl InsertNodesAliases {
    /// Returns the built `InsertNodesQuery` object.
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    /// List of lists of `key_values` to be inserted into the aliased nodes.
    /// The number of lists mut be the same as number of aliases.
    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }

    /// List of `key_values` to be inserted into the all nodes that are being created.
    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}

impl InsertNodesCount {
    /// Returns the built `InsertNodesQuery` object.
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    /// List of `key_values` to be inserted into the all nodes that are being created.
    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}

impl InsertNodesValues {
    /// Returns the built `InsertNodesQuery` object.
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}

impl InsertNodes {
    /// A list of `names` of the inserted nodes that will work as aliases
    /// instead of the numerical ids.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().nodes().aliases("a").query();
    /// QueryBuilder::insert().nodes().aliases("a").values(vec![vec![("k", 1).into()]]);
    /// QueryBuilder::insert().nodes().aliases("a").values_uniform(vec![("k", 1).into()]);
    /// ```
    pub fn aliases<T: Into<QueryAliases>>(mut self, names: T) -> InsertNodesAliases {
        self.0.aliases = Into::<QueryAliases>::into(names).0;

        InsertNodesAliases(self.0)
    }

    /// Number of nodes to insert.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().nodes().count(1).query();
    /// QueryBuilder::insert().nodes().count(1).values_uniform(vec![("k", 1).into()]);
    /// ```
    pub fn count(mut self, num: u64) -> InsertNodesCount {
        self.0.count = num;

        InsertNodesCount(self.0)
    }

    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertNodesIds {
        self.0.ids = ids.into();

        InsertNodesIds(self.0)
    }

    /// List of lists of `key_values` to be inserted into the nodes. The number of lists
    /// will be number created nodes.
    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}

impl InsertNodesIds {
    /// A list of `names` of the inserted nodes that will work as aliases
    /// instead of the numerical ids.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().nodes().aliases("a").query();
    /// QueryBuilder::insert().nodes().aliases("a").values(vec![vec![("k", 1).into()]]);
    /// QueryBuilder::insert().nodes().aliases("a").values_uniform(vec![("k", 1).into()]);
    /// ```
    pub fn aliases<T: Into<QueryAliases>>(mut self, names: T) -> InsertNodesAliases {
        self.0.aliases = Into::<QueryAliases>::into(names).0;

        InsertNodesAliases(self.0)
    }

    /// Number of nodes to insert.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().nodes().count(1).query();
    /// QueryBuilder::insert().nodes().count(1).values_uniform(vec![("k", 1).into()]);
    /// ```
    pub fn count(mut self, num: u64) -> InsertNodesCount {
        self.0.count = num;

        InsertNodesCount(self.0)
    }

    /// List of lists of `key_values` to be inserted into the nodes. The number of lists
    /// will be number created nodes.
    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}
