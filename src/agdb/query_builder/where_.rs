use super::where_key::WhereKey;
use super::where_logic_operator::WhereLogicOperator;
use crate::query::query_condition::direction::Direction;
use crate::query::query_condition::edge_count_condition::EdgeCountCondition;
use crate::query::query_condition::QueryCondition;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;

pub struct Where(pub SearchQuery);

impl Where {
    pub fn distance(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0.conditions.push(QueryCondition::Distance(comparison));

        WhereLogicOperator(self.0)
    }

    pub fn edge(mut self) -> WhereLogicOperator {
        self.0.conditions.push(QueryCondition::Edge);

        WhereLogicOperator(self.0)
    }

    pub fn edge_count(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0
            .conditions
            .push(QueryCondition::EdgeCount(EdgeCountCondition {
                comparison,
                direction: Direction::Both,
            }));

        WhereLogicOperator(self.0)
    }

    pub fn edge_count_from(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0
            .conditions
            .push(QueryCondition::EdgeCount(EdgeCountCondition {
                comparison,
                direction: Direction::From,
            }));

        WhereLogicOperator(self.0)
    }

    pub fn edge_count_to(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0
            .conditions
            .push(QueryCondition::EdgeCount(EdgeCountCondition {
                comparison,
                direction: Direction::To,
            }));

        WhereLogicOperator(self.0)
    }

    pub fn ids(mut self, ids: &[QueryId]) -> WhereLogicOperator {
        self.0
            .conditions
            .push(QueryCondition::Ids(QueryIds::Ids(ids.to_vec())));

        WhereLogicOperator(self.0)
    }

    pub fn key(self, key: DbKey) -> WhereKey {
        WhereKey {
            key,
            search: self.0,
        }
    }

    pub fn keys(mut self, names: &[DbKey]) -> WhereLogicOperator {
        self.0.conditions.push(QueryCondition::Keys(names.to_vec()));

        WhereLogicOperator(self.0)
    }

    pub fn node(mut self) -> WhereLogicOperator {
        self.0.conditions.push(QueryCondition::Node);

        WhereLogicOperator(self.0)
    }

    pub fn not(mut self) -> Self {
        self.0.conditions.push(QueryCondition::Not);

        self
    }

    pub fn not_beyond(mut self) -> Self {
        self.0.conditions.push(QueryCondition::NotBeyond);

        self
    }

    pub fn where_(mut self) -> Self {
        self.0.conditions.push(QueryCondition::Where);

        self
    }
}
