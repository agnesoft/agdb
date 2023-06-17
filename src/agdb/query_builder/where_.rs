use crate::query::query_condition::CountComparison;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionData;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;
use crate::Comparison;
use crate::DbKey;

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

    pub fn distance(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Distance { value: comparison },
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
            data: QueryConditionData::EdgeCount { value: comparison },
        });

        WhereLogicOperator(self)
    }

    pub fn edge_count_from(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountFrom { value: comparison },
        });

        WhereLogicOperator(self)
    }

    pub fn edge_count_to(mut self, comparison: CountComparison) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountTo { value: comparison },
        });

        WhereLogicOperator(self)
    }

    pub fn ids(mut self, ids: &[QueryId]) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Ids {
                values: ids.to_vec(),
            },
        });

        WhereLogicOperator(self)
    }

    pub fn key(self, key: DbKey) -> WhereKey {
        WhereKey { key, where_: self }
    }

    pub fn keys(mut self, names: &[DbKey]) -> WhereLogicOperator {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Keys {
                values: names.to_vec(),
            },
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
        self.query.conditions.push(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Where { conditions: vec![] },
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
        if self.0.conditions.len() > 1 {
            if let Some(last_conditions) = self.0.conditions.pop() {
                if let Some(current_conditions) = self.0.conditions.last_mut() {
                    if let Some(QueryCondition {
                        logic: _,
                        modifier: _,
                        data: QueryConditionData::Where { conditions },
                    }) = current_conditions.last_mut()
                    {
                        *conditions = last_conditions
                    }
                }
            }
        }

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

    pub fn query(self) -> SearchQuery {
        self.0.query
    }
}
