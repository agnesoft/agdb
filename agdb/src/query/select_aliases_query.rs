use crate::DbElement;
use crate::DbError;
use crate::DbImpl;
use crate::Query;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;
use crate::query_builder::search::SearchQueryBuilder;

/// Query to select aliases of given ids. All of the ids
/// must exist in the database and have an alias.
///
/// The result will be number of returned aliases and list
/// of elements with a single property `String("alias")` holding
/// the value `String`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDefImpl))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectAliasesQuery(pub QueryIds);

impl Query for SelectAliasesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        let mut result = QueryResult::default();

        match &self.0 {
            QueryIds::Ids(ids) => {
                result.elements.reserve(ids.len());
                result.result += ids.len() as i64;

                for id in ids {
                    match id {
                        QueryId::Id(db_id) => result.elements.push(DbElement {
                            id: *db_id,
                            from: None,
                            to: None,
                            values: vec![("alias", db.alias(*db_id)?).into()],
                        }),
                        QueryId::Alias(alias) => result.elements.push(DbElement {
                            id: db.db_id(id)?,
                            from: None,
                            to: None,
                            values: vec![("alias", alias).into()],
                        }),
                    }
                }
            }
            QueryIds::Search(search_query) => {
                for db_id in search_query.search(db)? {
                    if let Ok(alias) = db.alias(db_id) {
                        result.elements.push(DbElement {
                            id: db_id,
                            from: None,
                            to: None,
                            values: vec![("alias", alias).into()],
                        });
                    }
                }

                result.result = result.elements.len() as i64;
            }
        }

        Ok(result)
    }
}

impl Query for &SelectAliasesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}

impl SearchQueryBuilder for SelectAliasesQuery {
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
        SelectAliasesQuery(QueryIds::Ids(vec![])).search_mut();
    }
}
