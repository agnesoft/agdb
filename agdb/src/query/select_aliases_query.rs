use crate::DbElement;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryResult;
use crate::StorageData;

/// Query to select aliases of given ids. All of the ids
/// must exist in the database and have an alias.
///
/// The result will be number of returned aliases and list
/// of elements with a single property `String("alias")` holding
/// the value `String`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectAliasesQuery(pub QueryIds);

impl Query for SelectAliasesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
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
