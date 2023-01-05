use super::select_id_from::SelectIdFrom;
use super::select_id_to::SelectIdTo;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct SelectId(pub SearchQuery);

impl SelectId {
    pub fn from(mut self, id: QueryId) -> SelectIdFrom {
        self.0.origin = id;

        SelectIdFrom(self.0)
    }

    pub fn to(mut self, id: QueryId) -> SelectIdTo {
        self.0.destination = id;

        SelectIdTo(self.0)
    }
}
