use super::where_::Where;
use crate::db::db_key_order::DbKeyOrders;
use crate::query::query_condition::KeyValueComparison;
use crate::Comparison;
use crate::DbValue;
use crate::QueryCondition;
use crate::QueryConditionData;
use crate::QueryConditionLogic;
use crate::QueryConditionModifier;
use crate::QueryId;
use crate::SearchQuery;
use crate::SearchQueryAlgorithm;

pub trait SearchQueryBuilder {
    fn search_mut(&mut self) -> &mut SearchQuery;
}

/// Search builder query.
pub struct Search<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose search origin
/// and other parameters.
pub struct SearchFrom<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose search destination
/// and other parameters.
pub struct SearchTo<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose an index to search
/// instead of the graph search.
pub struct SearchIndex<T: SearchQueryBuilder> {
    pub index: DbValue,
    pub query: T,
}

/// Search builder query that lets you choose a a value to find
/// in the index.
pub struct SearchIndexValue<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose limit and offset.
pub struct SearchOrderBy<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose conditions.
pub struct SelectLimit<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose limit.
pub struct SelectOffset<T: SearchQueryBuilder>(pub T);

/// Search builder query that lets you choose search origin
/// and other parameters.
pub struct SearchAlgorithm<T: SearchQueryBuilder>(pub T);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl<T: SearchQueryBuilder> Search<T> {
    /// Use breadth-first (BFS) search algorithm. This option is redundant as
    /// BFS is the default. BFS means each level of the graph is examined in full
    /// before advancing to the next level. E.g. all edges coming from a node,
    /// then all the nodes connected to them in the same order. Then all edges
    /// coming from each of those nodes etc.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().breadth_first().from(1);
    /// QueryBuilder::search().breadth_first().to(1);
    /// ```
    pub fn breadth_first(mut self) -> SearchAlgorithm<T> {
        self.0.search_mut().algorithm = SearchQueryAlgorithm::BreadthFirst;
        SearchAlgorithm(self.0)
    }

    /// Use depth-first (DFS) search algorithm. DFS means each element is followed
    /// up to its dead end (or already visited element) before examining a next element.
    /// The algorithm is exhausting each path before backtracking one step at a time to
    /// try another when it reaches the end in any direction.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().depth_first().from(1);
    /// QueryBuilder::search().depth_first().to(1);
    /// ```
    pub fn depth_first(mut self) -> SearchAlgorithm<T> {
        self.0.search_mut().algorithm = SearchQueryAlgorithm::DepthFirst;
        SearchAlgorithm(self.0)
    }

    /// Searches all elements (nodes & edges) in the database disregarding the graph
    /// structure or any relationships between elements. This performs linear search
    /// through the entire database which may be prohibitively expensive. Consider
    /// using `limit()`.
    ///
    /// Note: While the full range of conitions can be used some conditions do not
    /// make logical sense (e.g. distance, beyond, edge_count etc.).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().elements().order_by([DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().elements().offset(5);
    /// QueryBuilder::search().elements().limit(10);
    /// QueryBuilder::search().elements().where_();
    /// ```
    pub fn elements(mut self) -> SearchTo<T> {
        self.0.search_mut().algorithm = SearchQueryAlgorithm::Elements;
        SearchTo(self.0)
    }

    /// Searches an index specified by `key`. This is to provide fast lookup of
    /// specific elements with particular key-value pair.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().index("k").value(1);
    /// ```
    pub fn index<K: Into<DbValue>>(self, key: K) -> SearchIndex<T> {
        SearchIndex {
            index: key.into(),
            query: self.0,
        }
    }

    /// Sets the origin of the search.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).query();
    /// QueryBuilder::search().from(1).to(2);
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().from(1).offset(5);
    /// QueryBuilder::search().from(1).limit(10);
    /// QueryBuilder::search().from(1).where_();
    /// ```
    pub fn from<I: Into<QueryId>>(mut self, id: I) -> SearchFrom<T> {
        self.0.search_mut().origin = id.into();
        SearchFrom(self.0)
    }

    /// Reverses the search setting only the destination. Reverse search
    /// follows the edges in reverse (to<-from).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().to(1).offset(5);
    /// QueryBuilder::search().to(1).limit(10);
    /// QueryBuilder::search().to(1).where_();
    /// ```
    pub fn to<I: Into<QueryId>>(mut self, id: I) -> SearchTo<T> {
        self.0.search_mut().destination = id.into();
        SearchTo(self.0)
    }
}

impl<T: SearchQueryBuilder> SearchAlgorithm<T> {
    /// Sets the origin of the search.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().depth_first().from(1).query();
    /// QueryBuilder::search().depth_first().from(1).to(2);
    /// QueryBuilder::search().depth_first().from(1).order_by([DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().depth_first().from(1).offset(5);
    /// QueryBuilder::search().depth_first().from(1).limit(10);
    /// QueryBuilder::search().depth_first().from(1).where_();
    /// ```
    pub fn from<I: Into<QueryId>>(mut self, id: I) -> SearchFrom<T> {
        self.0.search_mut().origin = id.into();
        SearchFrom(self.0)
    }

    /// Reverses the search setting only the destination. Reverse search
    /// follows the edges in reverse (to<-x<-from).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().depth_first().to(1).order_by([DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().depth_first().to(1).offset(5);
    /// QueryBuilder::search().depth_first().to(1).limit(10);
    /// QueryBuilder::search().depth_first().to(1).where_();
    /// ```
    pub fn to<I: Into<QueryId>>(mut self, id: I) -> SearchTo<T> {
        self.0.search_mut().destination = id.into();
        SearchTo(self.0)
    }
}

impl<T: SearchQueryBuilder> SearchFrom<T> {
    /// Sets the limit to number of ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is returned.
    /// However when doing a path search or requesting ordering of the result
    /// the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().from(1).limit(10).query();
    /// QueryBuilder::search().from(1).limit(10).where_();
    /// ```
    pub fn limit(mut self, value: u64) -> SelectLimit<T> {
        self.0.search_mut().limit = value;
        SelectLimit(self.0)
    }

    /// Sets the offset to the ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is
    /// returned. The `offset` ids will be skipped in the result.
    /// However when doing a path search or requesting ordering of the
    /// result the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().from(1).offset(10).query();
    /// QueryBuilder::search().from(1).offset(10).limit(5);
    /// QueryBuilder::search().from(1).offset(10).where_();
    /// ```
    pub fn offset(mut self, value: u64) -> SelectOffset<T> {
        self.0.search_mut().offset = value;
        SelectOffset(self.0)
    }

    /// Orders the result by `keys`.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).query();
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10);
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).limit(5);
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).where_();
    /// ```
    pub fn order_by<K: Into<DbKeyOrders>>(mut self, keys: K) -> SearchOrderBy<T> {
        self.0.search_mut().order_by = Into::<DbKeyOrders>::into(keys).0;
        SearchOrderBy(self.0)
    }

    /// Returns the built query object.
    pub fn query(self) -> T {
        self.0
    }

    /// Sets the destination (to) and changes the search algorithm to path search
    /// using the A* algorithm.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).to(2).query();
    /// QueryBuilder::search().from(1).to(2).order_by([DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().from(1).to(2).offset(10);
    /// QueryBuilder::search().from(1).to(2).limit(5);
    /// QueryBuilder::search().from(1).to(2).where_();
    /// ```
    pub fn to<I: Into<QueryId>>(mut self, id: I) -> SearchTo<T> {
        self.0.search_mut().destination = id.into();
        SearchTo(self.0)
    }

    /// Starts the condition builder.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().node();
    /// QueryBuilder::search().from(1).where_().edge();
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().ids(1);
    /// QueryBuilder::search().from(1).where_().keys("k");
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where<T> {
        Where::new(self.0)
    }
}

impl<T: SearchQueryBuilder> SearchOrderBy<T> {
    /// Sets the limit to number of ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is returned.
    /// However when doing a path search or requesting ordering of the result
    /// the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).limit(10).query();
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).limit(10).where_();
    /// ```
    pub fn limit(mut self, value: u64) -> SelectLimit<T> {
        self.0.search_mut().limit = value;
        SelectLimit(self.0)
    }

    /// Sets the offset to the ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is
    /// returned. The `offset` ids will be skipped in the result.
    /// However when doing a path search or requesting ordering of the
    /// result the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).query();
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).limit(5);
    /// QueryBuilder::search().from(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).where_();
    /// ```
    pub fn offset(mut self, value: u64) -> SelectOffset<T> {
        self.0.search_mut().offset = value;
        SelectOffset(self.0)
    }

    /// Returns the built query object.
    pub fn query(self) -> T {
        self.0
    }

    /// Starts the condition builder.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().node();
    /// QueryBuilder::search().from(1).where_().edge();
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().ids(1);
    /// QueryBuilder::search().from(1).where_().keys("k");
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where<T> {
        Where::new(self.0)
    }
}

impl<T: SearchQueryBuilder> SearchTo<T> {
    /// Sets the limit to number of ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is returned.
    /// However when doing a path search or requesting ordering of the result
    /// the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).limit(10).query();
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).limit(10).where_();
    /// ```
    pub fn limit(mut self, value: u64) -> SelectLimit<T> {
        self.0.search_mut().limit = value;
        SelectLimit(self.0)
    }

    /// Sets the offset to the ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is
    /// returned. The `offset` ids will be skipped in the result.
    /// However when doing a path search or requesting ordering of the
    /// result the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).query();
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).limit(5);
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).where_();
    /// ```
    pub fn offset(mut self, value: u64) -> SelectOffset<T> {
        self.0.search_mut().offset = value;
        SelectOffset(self.0)
    }

    /// Sets the offset to the ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is
    /// returned. The `offset` ids will be skipped in the result.
    /// However when doing a path search or requesting ordering of the
    /// result the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).query();
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).limit(5);
    /// QueryBuilder::search().to(1).order_by([DbKeyOrder::Asc("k".into())]).offset(10).where_();
    /// ```
    pub fn order_by<K: Into<DbKeyOrders>>(mut self, keys: K) -> SearchOrderBy<T> {
        self.0.search_mut().order_by = Into::<DbKeyOrders>::into(keys).0;
        SearchOrderBy(self.0)
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(self) -> T {
        self.0
    }

    /// Starts the condition builder.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().node();
    /// QueryBuilder::search().from(1).where_().edge();
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().ids(1);
    /// QueryBuilder::search().from(1).where_().keys("k");
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where<T> {
        Where::new(self.0)
    }
}

impl<T: SearchQueryBuilder> SelectLimit<T> {
    /// Returns the built query object.
    pub fn query(self) -> T {
        self.0
    }

    /// Starts the condition builder.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().node();
    /// QueryBuilder::search().from(1).where_().edge();
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().ids(1);
    /// QueryBuilder::search().from(1).where_().keys("k");
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where<T> {
        Where::new(self.0)
    }
}

impl<T: SearchQueryBuilder> SelectOffset<T> {
    /// Sets the limit to number of ids returned. If during the search
    /// the `limit + offset` is hit the search ends and the result is returned.
    /// However when doing a path search or requesting ordering of the result
    /// the search is first completed before the limit is applied.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).offset(10).limit(10).query();
    /// QueryBuilder::search().from(1).offset(10).limit(10).where_();
    /// ```
    pub fn limit(mut self, value: u64) -> SelectLimit<T> {
        self.0.search_mut().limit = value;
        SelectLimit(self.0)
    }

    /// Returns the built query object.
    pub fn query(self) -> T {
        self.0
    }

    /// Starts the condition builder.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, CountComparison};
    ///
    /// QueryBuilder::search().from(1).where_().node();
    /// QueryBuilder::search().from(1).where_().edge();
    /// QueryBuilder::search().from(1).where_().distance(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().ids(1);
    /// QueryBuilder::search().from(1).where_().keys("k");
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where<T> {
        Where::new(self.0)
    }
}

impl<T: SearchQueryBuilder> SearchIndex<T> {
    /// Sets the value to be searched in the index.
    pub fn value<V: Into<DbValue>>(mut self, value: V) -> SearchIndexValue<T> {
        self.query.search_mut().algorithm = SearchQueryAlgorithm::Index;
        self.query.search_mut().conditions.push(QueryCondition {
            data: QueryConditionData::KeyValue(KeyValueComparison {
                key: self.index,
                value: Comparison::Equal(value.into()),
            }),
            logic: QueryConditionLogic::And,
            modifier: QueryConditionModifier::None,
        });
        SearchIndexValue(self.query)
    }
}

impl<T: SearchQueryBuilder> SearchIndexValue<T> {
    /// Returns the built q object.
    pub fn query(self) -> T {
        self.0
    }
}
