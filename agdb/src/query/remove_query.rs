use crate::DbImpl;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;
use crate::query_builder::search::SearchQueryBuilder;

/// Query to remove database elements (nodes & edges). It
/// is not an error if any of the `ids` do not already exist.
///
/// All properties associated with a given element are also removed.
///
/// If removing nodes all of its incoming and outgoing edges are
/// also removed along with their properties.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct RemoveQuery(pub QueryIds);

impl QueryMut for RemoveQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        match &self.0 {
            QueryIds::Ids(ids) => {
                for id in ids {
                    if db.remove(id)? {
                        result.result -= 1;
                    }
                }
            }
            QueryIds::Search(search_query) => {
                for db_id in search_query.search(db)? {
                    if db.remove_id(db_id)? {
                        result.result -= 1;
                    }
                }
            }
        }

        Ok(result)
    }
}

impl QueryMut for &RemoveQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}

impl SearchQueryBuilder for RemoveQuery {
    fn search_mut(&mut self) -> &mut SearchQuery {
        if let QueryIds::Search(search) = &mut self.0 {
            search
        } else {
            panic!("Expected search query");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn missing_search() {
        RemoveQuery(QueryIds::Ids(vec![])).search_mut();
    }
}
