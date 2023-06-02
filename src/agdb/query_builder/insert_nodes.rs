use crate::db::db_key_value::DbKeyValue;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_values::QueryValues;

pub struct InsertNodes(pub InsertNodesQuery);

pub struct InsertNodesAliases(pub InsertNodesQuery);

pub struct InsertNodesCount(pub InsertNodesQuery);

pub struct InsertNodesValues(pub InsertNodesQuery);

impl InsertNodesAliases {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertNodesValues(self.0)
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertNodesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodesValues(self.0)
    }
}

impl InsertNodesCount {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertNodesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodesValues(self.0)
    }
}

impl InsertNodesValues {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}

impl InsertNodes {
    pub fn aliases(mut self, names: &[String]) -> InsertNodesAliases {
        self.0.aliases = names.to_vec();

        InsertNodesAliases(self.0)
    }

    pub fn count(mut self, num: u64) -> InsertNodesCount {
        self.0.count = num;

        InsertNodesCount(self.0)
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertNodesValues(self.0)
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertNodesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodesValues(self.0)
    }
}