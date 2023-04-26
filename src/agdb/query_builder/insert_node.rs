use crate::db::db_key_value::DbKeyValue;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;

pub struct InsertNode(pub InsertNodesQuery);

pub struct InsertNodeAlias(pub InsertNodesQuery);

pub struct InsertNodeValues(pub InsertNodesQuery);

pub struct InsertNodes(pub InsertNodesQuery);

pub struct InsertNodesAliases(pub InsertNodesQuery);

pub struct InsertNodesCount(pub InsertNodesQuery);

pub struct InsertNodesValues(pub InsertNodesQuery);

impl InsertNode {
    pub fn alias<T: ToString>(mut self, name: T) -> InsertNodeAlias {
        self.0.aliases.push(name.to_string());

        InsertNodeAlias(self.0)
    }

    pub fn values(mut self, key_values: &[DbKeyValue]) -> InsertNodeValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodeValues(self.0)
    }

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertNodeValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

        InsertNodeValues(self.0)
    }

    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}

impl InsertNodeAlias {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[DbKeyValue]) -> InsertNodeValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodeValues(self.0)
    }

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertNodeValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

        InsertNodeValues(self.0)
    }
}

impl InsertNodeValues {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}

impl InsertNodesAliases {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertNodesValues(self.0)
    }

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

        InsertNodesValues(self.0)
    }

    pub fn values_ids(mut self, ids: &[QueryId]) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(ids.to_vec()));

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

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

        InsertNodesValues(self.0)
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

    pub fn values_id<T: Into<QueryId>>(mut self, id: T) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id.into()]));

        InsertNodesValues(self.0)
    }

    pub fn values_ids(mut self, ids: &[QueryId]) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(ids.to_vec()));

        InsertNodesValues(self.0)
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertNodesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodesValues(self.0)
    }
}
