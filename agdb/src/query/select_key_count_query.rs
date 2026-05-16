use crate::DbElement;
use crate::DbError;
use crate::DbImpl;
use crate::Query;
use crate::QueryIds;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;
use crate::query_builder::search::SearchQueryBuilder;

/// Query to select number of properties (key count) of
/// given ids. All of the ids must exist in the database.
///
/// The result is the sum of all selected key counts.
/// The elements still contain individual
/// key counts in property `String("key_count")` as `u64`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectKeyCountQuery(pub QueryIds);

impl Query for SelectKeyCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        let mut result = QueryResult::default();
        let mut total_count = 0_u64;

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

        for id in db_ids {
            let key_count = db.key_count(id)?;
            total_count += key_count;
            result.elements.push(DbElement {
                id,
                from: db.from_id(id)?,
                to: db.to_id(id)?,
                values: vec![("key_count", key_count).into()],
            });
        }

        result.result = total_count;

        Ok(result)
    }
}

impl Query for &SelectKeyCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
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
