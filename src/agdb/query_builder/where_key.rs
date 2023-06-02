use super::where_logic_operator::WhereLogicOperator;
use crate::query::query_condition::key_value_condition::KeyValueCondition;
use crate::query::query_condition::QueryCondition;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;

pub struct WhereKey {
    pub key: DbKey,
    pub search: SearchQuery,
}

impl WhereKey {
    pub fn value(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.search
            .conditions
            .push(QueryCondition::KeyValue(KeyValueCondition {
                key: self.key,
                comparison,
            }));

        WhereLogicOperator(self.search)
    }
}
