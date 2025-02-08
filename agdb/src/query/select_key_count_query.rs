use crate::query_builder::search::SearchQueryBuilder;
use crate::DbElement;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;

/// Query to select number of properties (key count) of
/// given ids. All of the ids must exist in the database.
///
/// The result will be number of elements returned and the list
/// of elements with a single property `String("key_count")` with
/// a value `u64`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::api::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectKeyCountQuery(pub QueryIds);

impl Query for SelectKeyCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let db_ids = match &self.0 {
            QueryIds::Ids(ids) => {
                let mut db_ids = Vec::with_capacity(ids.len());

                for query_id in ids {
                    db_ids.push(db.db_id(query_id)?);
                }

                db_ids
            }
            QueryIds::Search(search_query) => search_query.search(db)?,
        };

        result.elements.reserve(db_ids.len());
        result.result = db_ids.len() as i64;

        for id in db_ids {
            result.elements.push(DbElement {
                id,
                from: db.from_id(id),
                to: db.to_id(id),
                values: vec![("key_count", db.key_count(id)?).into()],
            });
        }

        Ok(result)
    }
}

impl Query for &SelectKeyCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}

impl SearchQueryBuilder for SelectKeyCountQuery {
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
        SelectKeyCountQuery(QueryIds::Ids(vec![])).search_mut();
    }
}
