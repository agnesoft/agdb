use super::where_::Where;
use crate::db::db_key::DbKeyOrder;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;
use crate::SearchQueryAlgorithm;

/// Search builder query.
pub struct Search {}

/// Search builder query that lets you choose search origin
/// and other parameters.
pub struct SearchFrom(pub SearchQuery);

/// Search builder query that lets you choose search destination
/// and other parameters.
pub struct SearchTo(pub SearchQuery);

/// Search builder query that lets you choose limit and offset.
pub struct SearchOrderBy(pub SearchQuery);

/// Search builder query that lets you choose conditions.
pub struct SelectLimit(pub SearchQuery);

/// Search builder query that lets you choose limit.
pub struct SelectOffset(pub SearchQuery);

/// Search builder query that lets you choose search origin
/// and other parameters.
pub struct SearchAlgorithm(pub SearchQuery);

impl Search {
    /// Use breadth-first (BFS) search algorithm. This option is redundant as
    /// BFS is the default. BFS means each level of the graph is examined in full
    /// before advancing to the next level. E.g. all edges coming from a node,
    /// then all the nodes connected to them, then all edges coming from each of the
    /// nodes etc.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().breadth_first().from(1);
    /// QueryBuilder::search().breadth_first().to(1);
    /// ```
    pub fn breadth_first(self) -> SearchAlgorithm {
        SearchAlgorithm(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::from(0),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    /// Use depth-first (DFS) search algorithm. DFS means each element is followed
    /// up to its dead end (or already visited element) before examining a next element.
    /// E.g. first edge, its connected node, its first outgoing edge etc.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::search().depth_first().from(1);
    /// QueryBuilder::search().depth_first().to(1);
    /// ```
    pub fn depth_first(self) -> SearchAlgorithm {
        SearchAlgorithm(SearchQuery {
            algorithm: SearchQueryAlgorithm::DepthFirst,
            origin: QueryId::from(0),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
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
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().from(1).offset(5);
    /// QueryBuilder::search().from(1).limit(10);
    /// QueryBuilder::search().from(1).where_();
    /// ```
    pub fn from<T: Into<QueryId>>(self, id: T) -> SearchFrom {
        SearchFrom(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: id.into(),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    /// Reverses the search setting only the destination. Reverse search
    /// follows the edges in reverse (to<-from).
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().to(1).offset(5);
    /// QueryBuilder::search().to(1).limit(10);
    /// QueryBuilder::search().to(1).where_();
    /// ```
    pub fn to<T: Into<QueryId>>(self, id: T) -> SearchTo {
        SearchTo(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::from(0),
            destination: id.into(),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }
}

impl SearchAlgorithm {
    /// Sets the origin of the search.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().depth_first().from(1).query();
    /// QueryBuilder::search().depth_first().from(1).to(2);
    /// QueryBuilder::search().depth_first().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().depth_first().from(1).offset(5);
    /// QueryBuilder::search().depth_first().from(1).limit(10);
    /// QueryBuilder::search().depth_first().from(1).where_();
    /// ```
    pub fn from<T: Into<QueryId>>(mut self, id: T) -> SearchFrom {
        self.0.origin = id.into();
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
    /// QueryBuilder::search().depth_first().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().depth_first().to(1).offset(5);
    /// QueryBuilder::search().depth_first().to(1).limit(10);
    /// QueryBuilder::search().depth_first().to(1).where_();
    /// ```
    pub fn to<T: Into<QueryId>>(mut self, id: T) -> SearchTo {
        self.0.destination = id.into();
        SearchTo(self.0)
    }
}

impl SearchFrom {
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
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

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
    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

        SelectOffset(self.0)
    }

    /// Orders the result by `keys`.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::{QueryBuilder, DbKeyOrder};
    ///
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).query();
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10);
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).limit(5);
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).where_();
    /// ```
    pub fn order_by(mut self, keys: Vec<DbKeyOrder>) -> SearchOrderBy {
        self.0.order_by = keys;

        SearchOrderBy(self.0)
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(self) -> SearchQuery {
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
    /// QueryBuilder::search().from(1).to(2).order_by(vec![DbKeyOrder::Asc("k".into())]);
    /// QueryBuilder::search().from(1).to(2).offset(10);
    /// QueryBuilder::search().from(1).to(2).limit(5);
    /// QueryBuilder::search().from(1).to(2).where_();
    /// ```
    pub fn to<T: Into<QueryId>>(mut self, id: T) -> SearchTo {
        self.0.destination = id.into();

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
    /// QueryBuilder::search().from(1).where_().keys(vec!["k".into()]);
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SearchOrderBy {
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
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).limit(10).query();
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).limit(10).where_();
    /// ```
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

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
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).query();
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).limit(5);
    /// QueryBuilder::search().from(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).where_();
    /// ```
    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

        SelectOffset(self.0)
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(self) -> SearchQuery {
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
    /// QueryBuilder::search().from(1).where_().keys(vec!["k".into()]);
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SearchTo {
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
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).limit(10).query();
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).limit(10).where_();
    /// ```
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

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
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).query();
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).limit(5);
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).where_();
    /// ```
    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

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
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).query();
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).limit(5);
    /// QueryBuilder::search().to(1).order_by(vec![DbKeyOrder::Asc("k".into())]).offset(10).where_();
    /// ```
    pub fn order_by(mut self, keys: Vec<DbKeyOrder>) -> SearchOrderBy {
        self.0.order_by = keys;

        SearchOrderBy(self.0)
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(self) -> SearchQuery {
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
    /// QueryBuilder::search().from(1).where_().keys(vec!["k".into()]);
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SelectLimit {
    /// Returns the built `SearchQuery` object.
    pub fn query(self) -> SearchQuery {
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
    /// QueryBuilder::search().from(1).where_().keys(vec!["k".into()]);
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SelectOffset {
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
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    /// Returns the built `SearchQuery` object.
    pub fn query(self) -> SearchQuery {
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
    /// QueryBuilder::search().from(1).where_().keys(vec!["k".into()]);
    /// QueryBuilder::search().from(1).where_().key("k");
    /// QueryBuilder::search().from(1).where_().edge_count(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_from(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().edge_count_to(CountComparison::Equal(1));
    /// QueryBuilder::search().from(1).where_().where_();
    /// QueryBuilder::search().from(1).where_().not();
    /// QueryBuilder::search().from(1).where_().beyond();
    /// QueryBuilder::search().from(1).where_().not_beyond();
    /// ```
    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}
