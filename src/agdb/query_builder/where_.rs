use crate::query::query_condition::CountComparison;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionData;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query::query_values::QueryKeys;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;
use crate::QueryIds;

pub struct Where {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    conditions: Vec<Vec<QueryCondition>>,
    query: SearchQuery,
}

pub struct WhereKey {
    key: DbKey,
    where_: Where,
}

pub struct WhereLogicOperator(pub Where);

impl Where {
    pub fn new(query: SearchQuery) -> Self {
        Self {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: vec![vec![]],
            query,
        }
    }

    pub fn beyond(mut self) -> Where {
        self.modifier = QueryConditionModifier::Beyond;

        self
    }

    pub fn distance(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Distance(comparison),
        });

        WhereLogicOperator(self)
    }

    pub fn edge(mut self) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Edge,
        });

        WhereLogicOperator(self)
    }

    pub fn edge_count(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCount(comparison),
        });

        WhereLogicOperator(self)
    }

    pub fn edge_count_from(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountFrom(comparison),
        });

        WhereLogicOperator(self)
    }

    pub fn edge_count_to(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountTo(comparison),
        });

        WhereLogicOperator(self)
    }

    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Ids(Into::<QueryIds>::into(ids).get_ids()),
        });

        WhereLogicOperator(self)
    }

    pub fn key<T: Into<DbKey>>(self, key: T) -> WhereKey {
        WhereKey {
            key: key.into(),
            where_: self,
        }
    }

    pub fn keys<T: Into<QueryKeys>>(mut self, keys: T) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Keys(Into::<QueryKeys>::into(keys).0),
        });

        WhereLogicOperator(self)
    }

    pub fn node(mut self) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Node,
        });

        WhereLogicOperator(self)
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
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Where(vec![]),
        });
        self.conditions.push(vec![]);

        Self {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: self.conditions,
            query: self.query,
        }
    }

    fn add_condition(&mut self, condition: QueryCondition) {
        self.conditions.last_mut().unwrap().push(condition);
    }

    fn collapse_conditions(&mut self) -> bool {
        if self.conditions.len() > 1 {
            let last_conditions = self.conditions.pop().unwrap_or_default();
            let current_conditions = self.conditions.last_mut().unwrap();

            if let Some(QueryCondition {
                logic: _,
                modifier: _,
                data: QueryConditionData::Where(conditions),
            }) = current_conditions.last_mut()
            {
                *conditions = last_conditions;
                return true;
            }
        }

        false
    }
}

impl WhereKey {
    pub fn value(mut self, comparison: Comparison) -> WhereLogicOperator {
        let condition = QueryCondition {
            logic: self.where_.logic,
            modifier: self.where_.modifier,
            data: QueryConditionData::KeyValue {
                key: self.key,
                value: comparison,
            },
        };
        self.where_.add_condition(condition);
        WhereLogicOperator(self.where_)
    }
}

impl WhereLogicOperator {
    pub fn and(self) -> Where {
        Where {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: self.0.conditions,
            query: self.0.query,
        }
    }

    pub fn end_where(mut self) -> WhereLogicOperator {
        self.0.collapse_conditions();

        WhereLogicOperator(self.0)
    }

    pub fn or(self) -> Where {
        Where {
            logic: QueryConditionLogic::Or,
            modifier: QueryConditionModifier::None,
            conditions: self.0.conditions,
            query: self.0.query,
        }
    }

    pub fn query(mut self) -> SearchQuery {
        while self.0.collapse_conditions() {}
        std::mem::swap(&mut self.0.query.conditions, &mut self.0.conditions[0]);
        self.0.query
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::DbId;
    use crate::QueryId;
    use crate::SearchQueryAlgorithm;

    #[test]
    fn invalid_collapse() {
        let mut where_ = Where {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: vec![vec![], vec![]],
            query: SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(0)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            },
        };
        assert!(!where_.collapse_conditions());
    }
}
