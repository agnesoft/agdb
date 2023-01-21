use super::where_key::WhereKey;
use super::where_logic_operator::WhereLogicOperator;
use crate::query::condition::Condition;
use crate::query::direction::Direction;
use crate::query::edge_count_condition::EdgeCountCondition;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;

pub struct Where(pub SearchQuery);

impl Where {
    pub fn distance(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0.conditions.push(Condition::Distance(comparison));

        WhereLogicOperator(self.0)
    }

    pub fn edge(mut self) -> WhereLogicOperator {
        self.0.conditions.push(Condition::Edge);

        WhereLogicOperator(self.0)
    }

    pub fn edge_count(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0
            .conditions
            .push(Condition::EdgeCount(EdgeCountCondition {
                comparison,
                direction: Direction::Both,
            }));

        WhereLogicOperator(self.0)
    }

    pub fn edge_count_from(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0
            .conditions
            .push(Condition::EdgeCount(EdgeCountCondition {
                comparison,
                direction: Direction::From,
            }));

        WhereLogicOperator(self.0)
    }

    pub fn edge_count_to(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.0
            .conditions
            .push(Condition::EdgeCount(EdgeCountCondition {
                comparison,
                direction: Direction::To,
            }));

        WhereLogicOperator(self.0)
    }

    pub fn id(self, id: QueryId) -> WhereLogicOperator {
        self.ids(&[id])
    }

    pub fn ids(mut self, ids: &[QueryId]) -> WhereLogicOperator {
        self.0
            .conditions
            .push(Condition::Ids(QueryIds::Ids(ids.to_vec())));

        WhereLogicOperator(self.0)
    }

    pub fn key(self, key: DbKey) -> WhereKey {
        WhereKey {
            key,
            search: self.0,
        }
    }

    pub fn keys(mut self, names: &[DbKey]) -> WhereLogicOperator {
        self.0.conditions.push(Condition::Keys(names.to_vec()));

        WhereLogicOperator(self.0)
    }

    pub fn node(mut self) -> WhereLogicOperator {
        self.0.conditions.push(Condition::Node);

        WhereLogicOperator(self.0)
    }

    pub fn not(mut self) -> Self {
        self.0.conditions.push(Condition::Not);

        self
    }

    pub fn not_beyond(mut self) -> Self {
        self.0.conditions.push(Condition::NotBeyond);

        self
    }

    pub fn where_(mut self) -> Self {
        self.0.conditions.push(Condition::Where);

        self
    }
}
