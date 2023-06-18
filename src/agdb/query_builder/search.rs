use super::where_::Where;
use crate::db::db_key::DbKeyOrder;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct Search {}

pub struct SearchFrom(pub SearchQuery);

pub struct SearchTo(pub SearchQuery);

pub struct SearchOrderBy(pub SearchQuery);

pub struct SelectLimit(pub SearchQuery);

pub struct SelectOffset(pub SearchQuery);

impl Search {
    pub fn from<T: Into<QueryId>>(self, id: T) -> SearchFrom {
        SearchFrom(SearchQuery {
            origin: id.into(),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    pub fn to<T: Into<QueryId>>(self, id: T) -> SearchTo {
        SearchTo(SearchQuery {
            origin: QueryId::from(0),
            destination: id.into(),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }
}

impl SearchFrom {
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

        SelectOffset(self.0)
    }

    pub fn order_by(mut self, keys: &[DbKeyOrder]) -> SearchOrderBy {
        self.0.order_by = keys.to_vec();

        SearchOrderBy(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn to<T: Into<QueryId>>(mut self, id: T) -> SearchOrderBy {
        self.0.destination = id.into();

        SearchOrderBy(self.0)
    }

    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SearchOrderBy {
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

        SelectOffset(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SearchTo {
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

        SelectOffset(self.0)
    }

    pub fn order_by(mut self, keys: &[DbKeyOrder]) -> SearchOrderBy {
        self.0.order_by = keys.to_vec();

        SearchOrderBy(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }
}

impl SelectLimit {
    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}

impl SelectOffset {
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn where_(self) -> Where {
        Where::new(self.0)
    }
}
