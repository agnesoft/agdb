use super::where_::Where;
use crate::query::condition::Condition;
use crate::query::search_query::SearchQuery;

pub struct WhereLogicOperator(pub SearchQuery);

impl WhereLogicOperator {
    pub fn and(mut self) -> Where {
        self.0.conditions.push(Condition::And);

        Where(self.0)
    }

    pub fn end_where(mut self) -> WhereLogicOperator {
        self.0.conditions.push(Condition::EndWhere);

        WhereLogicOperator(self.0)
    }

    pub fn or(mut self) -> Where {
        self.0.conditions.push(Condition::Or);

        Where(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }
}
