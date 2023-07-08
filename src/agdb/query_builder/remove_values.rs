use crate::query::query_ids::QueryIds;
use crate::query::remove_values_query::RemoveValuesQuery;

/// Remove values builder that lets you select the ids from
/// which to remove the values.
pub struct RemoveValues(pub RemoveValuesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct RemoveValuesIds(pub RemoveValuesQuery);

impl RemoveValues {
    /// Id, list of ids or search of the database elements to delete
    /// the values from. All of the ids must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> RemoveValuesIds {
        self.0 .0.ids = ids.into();

        RemoveValuesIds(self.0)
    }
}

impl RemoveValuesIds {
    /// Returns the built `RemoveValuesQuery` object.
    pub fn query(self) -> RemoveValuesQuery {
        self.0
    }
}
