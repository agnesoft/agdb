use super::where_::Where;
use super::where_logic_operator::WhereLogicOperator;
use crate::query::condition::Condition;
use crate::query::key_value_condition::KeyValueCondition;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;

pub struct WhereKey {
    pub key: DbKey,
    pub search: SearchQuery,
}

impl WhereKey {
    pub fn and(mut self) -> Where {
        self.search.conditions.push(Condition::And);

        Where(self.search)
    }

    pub fn end_where(mut self) -> WhereLogicOperator {
        self.search.conditions.push(Condition::EndWhere);

        WhereLogicOperator(self.search)
    }

    pub fn or(mut self) -> Where {
        self.search.conditions.push(Condition::Or);

        Where(self.search)
    }

    pub fn query(mut self) -> SearchQuery {
        self.search.conditions.push(Condition::Keys(vec![self.key]));

        self.search
    }

    pub fn value(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.search
            .conditions
            .push(Condition::KeyValue(KeyValueCondition {
                key: self.key,
                comparison,
            }));

        WhereLogicOperator(self.search)
    }
}
