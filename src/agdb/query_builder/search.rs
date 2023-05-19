use super::where_::Where;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct Search {}

pub struct SearchFrom(pub SearchQuery);

pub struct SearchTo(pub SearchQuery);

pub struct SelectLimit(pub SearchQuery);

pub struct SelectOffset(pub SearchQuery);

impl Search {
    pub fn from(self, id: QueryId) -> SearchFrom {
        SearchFrom(SearchQuery {
            origin: id,
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    pub fn to(self, id: QueryId) -> SearchTo {
        SearchTo(SearchQuery {
            origin: QueryId::from(0),
            destination: id,
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

    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn to(mut self, id: QueryId) -> SearchTo {
        self.0.destination = id;

        SearchTo(self.0)
    }

    pub fn where_(self) -> Where {
        Where(self.0)
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

    pub fn query(self) -> SearchQuery {
        self.0
    }
}

impl SelectLimit {
    pub fn query(self) -> SearchQuery {
        self.0
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
}
