use super::where_::Where;
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
    pub fn and(mut self) -> Where {
        self.search.conditions.push(QueryCondition::And);

        Where(self.search)
    }

    pub fn end_where(mut self) -> WhereLogicOperator {
        self.search.conditions.push(QueryCondition::EndWhere);

        WhereLogicOperator(self.search)
    }

    pub fn or(mut self) -> Where {
        self.search.conditions.push(QueryCondition::Or);

        Where(self.search)
    }

    pub fn query(mut self) -> SearchQuery {
        self.search
            .conditions
            .push(QueryCondition::Keys(vec![self.key]));

        self.search
    }

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
