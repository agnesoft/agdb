use crate::query::query_condition::CountComparison;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;

pub struct Where {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    query: SearchQuery,
}

pub struct WhereKey {
    key: DbKey,
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    query: SearchQuery,
}

pub struct WhereLogicOperator(pub SearchQuery);

impl Where {
    pub fn new(query: SearchQuery) -> Self {
        Self {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            query,
        }
    }

    pub fn distance(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::Distance {
            logic: self.logic,
            modifier: self.modifier,
            value: comparison,
        });

        WhereLogicOperator(self.query)
    }

    pub fn edge(mut self) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::Edge {
            logic: self.logic,
            modifier: self.modifier,
        });

        WhereLogicOperator(self.query)
    }

    pub fn edge_count(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::EdgeCount {
            logic: self.logic,
            modifier: self.modifier,
            value: comparison,
        });

        WhereLogicOperator(self.query)
    }

    pub fn edge_count_from(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::EdgeCountFrom {
            logic: self.logic,
            modifier: self.modifier,
            value: comparison,
        });

        WhereLogicOperator(self.query)
    }

    pub fn edge_count_to(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::EdgeCountTo {
            logic: self.logic,
            modifier: self.modifier,
            value: comparison,
        });

        WhereLogicOperator(self.query)
    }

    pub fn ids(mut self, ids: &[QueryId]) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::Ids {
            logic: self.logic,
            modifier: self.modifier,
            values: QueryIds::Ids(ids.to_vec()),
        });

        WhereLogicOperator(self.query)
    }

    pub fn key(self, key: DbKey) -> WhereKey {
        WhereKey {
            key,
            logic: self.logic,
            modifier: self.modifier,
            query: self.query,
        }
    }

    pub fn keys(mut self, names: &[DbKey]) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::Keys {
            logic: self.logic,
            modifier: self.modifier,
            values: names.to_vec(),
        });

        WhereLogicOperator(self.query)
    }

    pub fn node(mut self) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::Node {
            logic: self.logic,
            modifier: self.modifier,
        });

        WhereLogicOperator(self.query)
    }

    pub fn not(mut self) -> Self {
        self.modifier = QueryConditionModifier::Not;

        self
    }

    pub fn not_beyond(mut self) -> Self {
        self.modifier = QueryConditionModifier::NotBeyond;

        self
    }

    pub fn where_(mut self) -> Self {
        self.query.conditions.push(QueryCondition::Where {
            logic: self.logic,
            modifier: self.modifier,
        });

        Self {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            query: self.query,
        }
    }
}

impl WhereKey {
    pub fn value(mut self, comparison: Comparison) -> WhereLogicOperator {
        self.query.conditions.push(QueryCondition::KeyValue {
            logic: self.logic,
            modifier: self.modifier,
            key: self.key,
            value: comparison,
        });

        WhereLogicOperator(self.query)
    }
}

impl WhereLogicOperator {
    pub fn and(self) -> Where {
        Where {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            query: self.0,
        }
    }

    pub fn end_where(mut self) -> WhereLogicOperator {
        self.0.conditions.push(QueryCondition::EndWhere);

        WhereLogicOperator(self.0)
    }

    pub fn or(self) -> Where {
        Where {
            logic: QueryConditionLogic::Or,
            modifier: QueryConditionModifier::None,
            query: self.0,
        }
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }
}
