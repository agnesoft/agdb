use crate::Comparison;
use crate::DbType;
use crate::DbValue;
use crate::QueryIds;
use crate::db::db_value::DbValues;
use crate::query::query_condition::CountComparison;
use crate::query::query_condition::KeyValueComparison;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionData;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query_builder::search::SearchQueryBuilder;
use crate::query_builder::where_::DB_ELEMENT_ID_KEY;

/// Condition builder for non-graph searches (e.g. index search).
/// This is a restricted version of [`Where`](crate::Where) that
/// excludes traversal-specific conditions that have no meaning
/// outside of a graph search: `distance`, `beyond`, `not_beyond`,
/// and `neighbor`.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct WhereFilter<T: SearchQueryBuilder> {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    conditions: Vec<Vec<QueryCondition>>,
    query: T,
}

/// Condition builder for `key` condition in non-graph searches.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct WhereFilterKey<T: SearchQueryBuilder> {
    key: DbValue,
    where_: WhereFilter<T>,
}

/// Condition builder setting the logic operator in non-graph searches.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct WhereFilterLogicOperator<T: SearchQueryBuilder>(pub WhereFilter<T>);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: SearchQueryBuilder> WhereFilter<T> {
    /// Only elements that are edges will pass this condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().index("k").value(1).where_().edge().query();
    /// ```
    pub fn edge(mut self) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Edge,
        });

        WhereFilterLogicOperator(self)
    }

    /// Only nodes can pass this condition and only if `edge_count`
    /// (from + to edges) is compared true against `comparison`. Note that self-referential
    /// edges are counted twice (e.g. node with an edge to itself will appear to have
    /// "2" edges, one outgoing and one incoming).
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().index("k").value(1).where_().edge_count(1).query();
    /// QueryBuilder::search().index("k").value(1).where_().edge_count(CountComparison::GreaterThan(1)).query();
    /// ```
    pub fn edge_count<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCount(comparison.into()),
        });

        WhereFilterLogicOperator(self)
    }

    /// Only nodes can pass this condition and only if `edge_count_from`
    /// (outgoing/from edges) is compared true against `comparison`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().index("k").value(1).where_().edge_count_from(1).query();
    /// QueryBuilder::search().index("k").value(1).where_().edge_count_from(CountComparison::GreaterThan(1)).query();
    /// ```
    pub fn edge_count_from<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountFrom(comparison.into()),
        });

        WhereFilterLogicOperator(self)
    }

    /// Only nodes can pass this condition and only if `edge_count_to`
    /// (incoming/to edges) is compared true against `comparison`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().index("k").value(1).where_().edge_count_to(1).query();
    /// QueryBuilder::search().index("k").value(1).where_().edge_count_to(CountComparison::GreaterThan(0)).query();
    /// ```
    pub fn edge_count_to<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountTo(comparison.into()),
        });

        WhereFilterLogicOperator(self)
    }

    /// Convenience condition. If `E` returns `Some` from DbType::db_element_id()
    /// it is equivalent to `key("db_element_id").value(element_id)` otherwise
    /// it is equivalent to `keys(E::db_keys())`.
    pub fn element<E: DbType>(self) -> WhereFilterLogicOperator<T> {
        if let Some(element_id) = E::db_element_id() {
            self.key(DB_ELEMENT_ID_KEY).value(element_id)
        } else {
            self.keys(E::db_keys())
        }
    }

    /// Only elements listed in `ids` will pass this condition.
    ///
    /// NOTE: Search query is NOT supported here and will be ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().index("k").value(1).where_().not().ids(1).query();
    /// ```
    pub fn ids<I: Into<QueryIds>>(mut self, ids: I) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Ids(Into::<QueryIds>::into(ids).get_ids()),
        });

        WhereFilterLogicOperator(self)
    }

    /// Initiates the `key` condition that tests the key for a
    /// particular value set in the next step. The value accepts comparison method.
    /// If a value is given without a method it will default to `Comparison::Equal`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, Comparison};
    ///
    /// QueryBuilder::search().index("k").value(1).where_().key("k").value(Comparison::LessThan(1.into())).query();
    /// QueryBuilder::search().index("k").value(1).where_().key("k").value(1).query();
    /// ```
    pub fn key<K: Into<DbValue>>(self, key: K) -> WhereFilterKey<T> {
        WhereFilterKey {
            key: key.into(),
            where_: self,
        }
    }

    /// Only elements with all properties listed in `keys` (regardless of values)
    /// will pass this condition ("all"). To achieve "any" you need to chain the
    /// `keys()` condition with `or()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().index("k").value(1).where_().keys("k").query();
    /// QueryBuilder::search().index("k").value(1).where_().keys("a").or().keys("b").query();
    /// ```
    pub fn keys<K: Into<DbValues>>(mut self, keys: K) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Keys(Into::<DbValues>::into(keys).0),
        });

        WhereFilterLogicOperator(self)
    }

    /// Only elements that are nodes will pass this condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().index("k").value(1).where_().node().query();
    /// ```
    pub fn node(mut self) -> WhereFilterLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Node,
        });

        WhereFilterLogicOperator(self)
    }

    /// Sets the condition modifier reversing the outcome of the following
    /// condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().index("k").value(1).where_().not().keys("k").query();
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Self {
        self.modifier = QueryConditionModifier::Not;

        self
    }

    /// Starts a sub-condition (it semantically represents an open bracket). The
    /// conditions in a sub-condition are collapsed into single condition when
    /// evaluated and passed to the previous level.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search()
    ///   .index("k")
    ///   .value(1)
    ///   .where_()
    ///   .node()
    ///   .and()
    ///   .where_()
    ///   .keys("a")
    ///   .or()
    ///   .keys("b")
    ///   .end_where()
    ///   .query();
    /// ```
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

    /// Shortcut for `.edge_count(CountComparison::GreaterThan(v))`.
    pub fn edge_count_greater_than(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count(CountComparison::GreaterThan(v))
    }

    /// Shortcut for `.edge_count(CountComparison::GreaterThanOrEqual(v))`.
    pub fn edge_count_greater_than_or_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count(CountComparison::GreaterThanOrEqual(v))
    }

    /// Shortcut for `.edge_count(CountComparison::LessThan(v))`.
    pub fn edge_count_less_than(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count(CountComparison::LessThan(v))
    }

    /// Shortcut for `.edge_count(CountComparison::LessThanOrEqual(v))`.
    pub fn edge_count_less_than_or_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count(CountComparison::LessThanOrEqual(v))
    }

    /// Shortcut for `.edge_count(CountComparison::NotEqual(v))`.
    pub fn edge_count_not_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count(CountComparison::NotEqual(v))
    }

    /// Shortcut for `.edge_count_from(CountComparison::GreaterThan(v))`.
    pub fn edge_count_from_greater_than(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_from(CountComparison::GreaterThan(v))
    }

    /// Shortcut for `.edge_count_from(CountComparison::GreaterThanOrEqual(v))`.
    pub fn edge_count_from_greater_than_or_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_from(CountComparison::GreaterThanOrEqual(v))
    }

    /// Shortcut for `.edge_count_from(CountComparison::LessThan(v))`.
    pub fn edge_count_from_less_than(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_from(CountComparison::LessThan(v))
    }

    /// Shortcut for `.edge_count_from(CountComparison::LessThanOrEqual(v))`.
    pub fn edge_count_from_less_than_or_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_from(CountComparison::LessThanOrEqual(v))
    }

    /// Shortcut for `.edge_count_from(CountComparison::NotEqual(v))`.
    pub fn edge_count_from_not_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_from(CountComparison::NotEqual(v))
    }

    /// Shortcut for `.edge_count_to(CountComparison::GreaterThan(v))`.
    pub fn edge_count_to_greater_than(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_to(CountComparison::GreaterThan(v))
    }

    /// Shortcut for `.edge_count_to(CountComparison::GreaterThanOrEqual(v))`.
    pub fn edge_count_to_greater_than_or_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_to(CountComparison::GreaterThanOrEqual(v))
    }

    /// Shortcut for `.edge_count_to(CountComparison::LessThan(v))`.
    pub fn edge_count_to_less_than(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_to(CountComparison::LessThan(v))
    }

    /// Shortcut for `.edge_count_to(CountComparison::LessThanOrEqual(v))`.
    pub fn edge_count_to_less_than_or_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_to(CountComparison::LessThanOrEqual(v))
    }

    /// Shortcut for `.edge_count_to(CountComparison::NotEqual(v))`.
    pub fn edge_count_to_not_equal(self, v: u64) -> WhereFilterLogicOperator<T> {
        self.edge_count_to(CountComparison::NotEqual(v))
    }

    pub(crate) fn new(query: T) -> Self {
        Self {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: vec![vec![]],
            query,
        }
    }

    fn add_condition(&mut self, condition: QueryCondition) {
        self.conditions
            .last_mut()
            .expect("Conditions should not be empty")
            .push(condition);
    }

    fn collapse_conditions(&mut self) -> bool {
        if self.conditions.len() > 1 {
            let last_conditions = self.conditions.pop().unwrap_or_default();
            let current_conditions = self
                .conditions
                .last_mut()
                .expect("Expected conditions of length of at least 2");

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

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: SearchQueryBuilder> WhereFilterKey<T> {
    /// Sets the value of the `key` condition to `comparison`. Taking comparison method. If
    /// a value is provided without a method it will default to `Comparison::Equal`).
    pub fn value<Comp: Into<Comparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereFilterLogicOperator<T> {
        let condition = QueryCondition {
            logic: self.where_.logic,
            modifier: self.where_.modifier,
            data: QueryConditionData::KeyValue(KeyValueComparison {
                key: self.key,
                value: comparison.into(),
            }),
        };
        self.where_.add_condition(condition);
        WhereFilterLogicOperator(self.where_)
    }

    /// Shortcut for `.value(Comparison::GreaterThan(v.into()))`.
    pub fn greater_than<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::GreaterThan(v.into()))
    }

    /// Shortcut for `.value(Comparison::GreaterThanOrEqual(v.into()))`.
    pub fn greater_than_or_equal<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::GreaterThanOrEqual(v.into()))
    }

    /// Shortcut for `.value(Comparison::LessThan(v.into()))`.
    pub fn less_than<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::LessThan(v.into()))
    }

    /// Shortcut for `.value(Comparison::LessThanOrEqual(v.into()))`.
    pub fn less_than_or_equal<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::LessThanOrEqual(v.into()))
    }

    /// Shortcut for `.value(Comparison::NotEqual(v.into()))`.
    pub fn not_equal<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::NotEqual(v.into()))
    }

    /// Shortcut for `.value(Comparison::Contains(v.into()))`.
    pub fn contains<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::Contains(v.into()))
    }

    /// Shortcut for `.value(Comparison::StartsWith(v.into()))`.
    pub fn starts_with<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::StartsWith(v.into()))
    }

    /// Shortcut for `.value(Comparison::EndsWith(v.into()))`.
    pub fn ends_with<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::EndsWith(v.into()))
    }

    /// Shortcut for `.value(Comparison::Any(v.into()))`.
    pub fn any<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::Any(v.into()))
    }

    /// Shortcut for `.value(Comparison::Regex(v.into()))`.
    pub fn regex<V: Into<DbValue>>(self, v: V) -> WhereFilterLogicOperator<T> {
        self.value(Comparison::Regex(v.into()))
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: SearchQueryBuilder> WhereFilterLogicOperator<T> {
    /// Sets the logic operator for the following condition
    /// to logical AND (&&). The condition passes only if
    /// both sides evaluates to `true`.
    pub fn and(self) -> WhereFilter<T> {
        WhereFilter {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: self.0.conditions,
            query: self.0.query,
        }
    }

    /// Closes the current level condition level returning
    /// to the previous one. It semantically represents a
    /// closing bracket.
    pub fn end_where(mut self) -> WhereFilterLogicOperator<T> {
        self.0.collapse_conditions();

        WhereFilterLogicOperator(self.0)
    }

    /// Sets the logic operator for the following condition
    /// to logical OR (||). The condition passes if
    /// either side evaluates to `true`.
    pub fn or(self) -> WhereFilter<T> {
        WhereFilter {
            logic: QueryConditionLogic::Or,
            modifier: QueryConditionModifier::None,
            conditions: self.0.conditions,
            query: self.0.query,
        }
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(mut self) -> T {
        while self.0.collapse_conditions() {}

        // Append the filter conditions after the existing ones (the index
        // condition must remain first for the Index search algorithm).
        self.0
            .query
            .search_mut()
            .conditions
            .append(&mut self.0.conditions[0]);

        self.0.query
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::SearchQuery;

    #[test]
    fn invalid_collapse() {
        let mut where_ = WhereFilter::<SearchQuery> {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: vec![vec![], vec![]],
            query: SearchQuery::new(),
        };
        assert!(!where_.collapse_conditions());
    }
}
