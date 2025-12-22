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

pub const DB_ELEMENT_ID_KEY: &str = "db_element_id";

/// Condition builder
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct Where<T: SearchQueryBuilder> {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    conditions: Vec<Vec<QueryCondition>>,
    query: T,
}

/// Condition builder for `key` condition.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct WhereKey<T: SearchQueryBuilder> {
    key: DbValue,
    where_: Where<T>,
}

/// Condition builder setting the logic operator.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct WhereLogicOperator<T: SearchQueryBuilder>(pub Where<T>);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: SearchQueryBuilder> Where<T> {
    /// Sets the condition modifier for the following condition so
    /// that the search will continue beyond current element only
    /// if the condition is satisfied. For the opposite effect see
    /// `not_beyond()`.
    ///
    /// NOTE: This condition applies to the starting element as well.
    /// A common issue is that the starting element is not followed
    /// as it may not pass this condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// // Only elements with `k` key will be followed during search.
    /// QueryBuilder::search().from(1).where_().beyond().keys("k").query();
    ///
    /// // Only edges or nodes with exactly 1 edge are followed.
    /// QueryBuilder::search().from(1).where_().beyond().edge().or().edge_count(1);
    /// ```
    pub fn beyond(mut self) -> Self {
        self.modifier = QueryConditionModifier::Beyond;
        self
    }

    /// Sets the distance condition. It can be used both for accepting
    /// elements during search and for limiting the area of the search
    /// on the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// // Search adjacent nodes that are exactly at distance 2 (neighbors) using a shorthand
    /// QueryBuilder::search().from(1).where_().neighbor().query();
    ///
    /// // Search at most to distance 2 (1 = first edge, 2 = neighbouring node)
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::LessThanOrEqual(2)).query();
    ///
    /// // Start accepting elements at distance greater than 1 (2+)
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::GreaterThan(1)).query();
    /// ```
    pub fn distance<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Distance(comparison.into()),
        });

        WhereLogicOperator(self)
    }

    /// Only elements that are edges will pass this condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().edge().query();
    /// ```
    pub fn edge(mut self) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Edge,
        });

        WhereLogicOperator(self)
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
    /// QueryBuilder::search().from(1).where_().edge_count(1).query(); // same as CountComparison::Equal(1)
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::GreaterThan(1)).query();
    /// ```
    pub fn edge_count<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCount(comparison.into()),
        });

        WhereLogicOperator(self)
    }

    /// Only nodes can pass this condition and only if `edge_count_from`
    /// (outgoing/from edges) is compared true against `comparison`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().edge_count_from(1).query(); // same as CountComparison::Equal(1)
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::GreaterThan(1)).query();
    /// ```
    pub fn edge_count_from<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountFrom(comparison.into()),
        });

        WhereLogicOperator(self)
    }

    /// Only nodes can pass this condition and only if `edge_count_to`
    /// (incoming/to edges) is compared true against `comparison`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().edge_count_to(1).query(); // same as CountComparison::Equal(1)
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::GreaterThan(0)).query();
    /// ```
    pub fn edge_count_to<Comp: Into<CountComparison>>(
        mut self,
        comparison: Comp,
    ) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::EdgeCountTo(comparison.into()),
        });

        WhereLogicOperator(self)
    }

    /// Convenience condition. If `E` returns `Some` from DbType::db_element_id()
    /// it is equivalent to `key("db_element_id").value(element_id)` otherwise
    /// it is equivalent to `keys(E::db_keys())`.
    pub fn element<E: DbType>(self) -> WhereLogicOperator<T> {
        if let Some(element_id) = E::db_element_id() {
            self.key(DB_ELEMENT_ID_KEY).value(element_id)
        } else {
            self.keys(E::db_keys())
        }
    }

    /// Only elements listed in `ids` will pass this condition. It is usually combined
    /// with a modifier like `not_beyond()` or `not()`.
    ///
    /// NOTE: Search query is NOT supported here and will be ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// // Exclude element 1 from result (the starting element)
    /// QueryBuilder::search().from(1).where_().not().ids(1).query();
    ///
    /// // Do not continue the search beyond "alias" element
    /// QueryBuilder::search().from(1).where_().not_beyond().ids("alias").query();
    /// ```
    pub fn ids<I: Into<QueryIds>>(mut self, ids: I) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Ids(Into::<QueryIds>::into(ids).get_ids()),
        });

        WhereLogicOperator(self)
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
    /// // Includes only elements with property `String("k") == 1_i64`
    /// QueryBuilder::search().from(1).where_().key("k").value(Comparison::LessThan(1.into())).query();
    ///
    /// // Same as the `key("k").value(Comparison::Equal(1.into()))`
    /// QueryBuilder::search().from(1).where_().key("k").value(1).query();
    /// ```
    pub fn key<K: Into<DbValue>>(self, key: K) -> WhereKey<T> {
        WhereKey {
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
    /// // Include only elements with "k" property (key)
    /// QueryBuilder::search().from(1).where_().keys("k").query();
    ///
    /// // Includes only elements with either "a" or "b" properties (keys).
    /// QueryBuilder::search().from(1).where_().keys("a").or().keys("b").query();
    /// ```
    pub fn keys<K: Into<DbValues>>(mut self, keys: K) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Keys(Into::<DbValues>::into(keys).0),
        });

        WhereLogicOperator(self)
    }

    /// Convenience shorthand to select neighboring elements
    /// equivalient to `distance(CountComparison::Equal(2))`.
    pub fn neighbor(self) -> WhereLogicOperator<T> {
        self.distance(CountComparison::Equal(2))
    }

    /// Only elements that are nodes will pass this condition.
    ///    
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().node().query();
    /// ```
    pub fn node(mut self) -> WhereLogicOperator<T> {
        self.add_condition(QueryCondition {
            logic: self.logic,
            modifier: self.modifier,
            data: QueryConditionData::Node,
        });

        WhereLogicOperator(self)
    }

    /// Sets the condition modifier reversing the outcome of the following
    /// condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// // Includes elements WITHOUT the "k" property (key).
    /// QueryBuilder::search().from(1).where_().not().keys("k").query();
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Self {
        self.modifier = QueryConditionModifier::Not;

        self
    }

    /// Sets the condition modifier for the following condition so
    /// that the search will NOT continue beyond current element only
    /// if the condition IS satisfied. For the opposite effect see
    /// `beyond()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// // Elements with `k` key will NOT be followed during search.
    /// QueryBuilder::search().from(1).where_().not_beyond().keys("k").query();
    ///
    /// // Elements 1 and 2 will NOT be followed during search.
    /// QueryBuilder::search().from(1).where_().not_beyond().ids([1, 2]);
    /// ```
    pub fn not_beyond(mut self) -> Self {
        self.modifier = QueryConditionModifier::NotBeyond;

        self
    }

    /// Starts a sub-condition (it semantically represents an open bracket). The
    /// conditions in a sub-condition are collapsed into single condition when
    /// evaluated and passed to the previous level. Any condition modifiers can still
    /// apply to the collapsed condition if applied on the `where_()` condition.
    ///
    /// # Examples
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// // Select only neighbor elements (= nodes)
    /// // but only follow elements with "k" property
    /// // or nodes (this is to follow the starting node)
    /// QueryBuilder::search()
    ///   .from(1)
    ///   .where_()
    ///   .neighbor()
    ///   .and()
    ///   .beyond()
    ///   .where_()
    ///   .keys("k")
    ///   .or()
    ///   .node()
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
impl<T: SearchQueryBuilder> WhereKey<T> {
    /// Sets the value of the `key` condition to `comparison`. Taking comparison method. If
    /// a value is provided without a method it will default to `Comparison::Equal`).
    pub fn value<Comp: Into<Comparison>>(mut self, comparison: Comp) -> WhereLogicOperator<T> {
        let condition = QueryCondition {
            logic: self.where_.logic,
            modifier: self.where_.modifier,
            data: QueryConditionData::KeyValue(KeyValueComparison {
                key: self.key,
                value: comparison.into(),
            }),
        };
        self.where_.add_condition(condition);
        WhereLogicOperator(self.where_)
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: SearchQueryBuilder> WhereLogicOperator<T> {
    /// Sets the logic operator for the following condition
    /// to logical AND (&&). The condition passes only if
    /// both sides evaluates to `true`.
    pub fn and(self) -> Where<T> {
        Where {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: self.0.conditions,
            query: self.0.query,
        }
    }

    /// Closes the current level condition level returning
    /// to the previous one. It semantically represents a
    /// closing bracket.
    pub fn end_where(mut self) -> WhereLogicOperator<T> {
        self.0.collapse_conditions();

        WhereLogicOperator(self.0)
    }

    /// Sets the logic operator for the following condition
    /// to logical OR (||). The condition passes only if
    /// both sides evaluates to `false`.
    pub fn or(self) -> Where<T> {
        Where {
            logic: QueryConditionLogic::Or,
            modifier: QueryConditionModifier::None,
            conditions: self.0.conditions,
            query: self.0.query,
        }
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(mut self) -> T {
        while self.0.collapse_conditions() {}

        if !self.0.query.search_mut().conditions.is_empty() {
            let existing_conditions = std::mem::take(&mut self.0.query.search_mut().conditions);
            self.0.conditions[0].extend(existing_conditions);
        }

        std::mem::swap(
            &mut self.0.query.search_mut().conditions,
            &mut self.0.conditions[0],
        );

        self.0.query
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::SearchQuery;

    #[test]
    fn invalid_collapse() {
        let mut where_ = Where::<SearchQuery> {
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
            conditions: vec![vec![], vec![]],
            query: SearchQuery::new(),
        };
        assert!(!where_.collapse_conditions());
    }
}
