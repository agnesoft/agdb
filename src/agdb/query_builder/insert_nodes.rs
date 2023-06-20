use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_values::{MultiValues, QueryValues, SingleValues};

pub struct InsertNodes(pub InsertNodesQuery);

pub struct InsertNodesAliases(pub InsertNodesQuery);

pub struct InsertNodesCount(pub InsertNodesQuery);

pub struct InsertNodesValues(pub InsertNodesQuery);

impl InsertNodesAliases {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }

    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}

impl InsertNodesCount {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}

impl InsertNodesValues {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}

impl InsertNodes {
    pub fn aliases<T: Into<QueryAliases>>(mut self, names: T) -> InsertNodesAliases {
        self.0.aliases = Into::<QueryAliases>::into(names).0;

        InsertNodesAliases(self.0)
    }

    pub fn count(mut self, num: u64) -> InsertNodesCount {
        self.0.count = num;

        InsertNodesCount(self.0)
    }

    pub fn values<T: Into<MultiValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(Into::<MultiValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }

    pub fn values_uniform<T: Into<SingleValues>>(mut self, key_values: T) -> InsertNodesValues {
        self.0.values = QueryValues::Single(Into::<SingleValues>::into(key_values).0);

        InsertNodesValues(self.0)
    }
}
