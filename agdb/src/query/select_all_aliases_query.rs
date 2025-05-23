use crate::DbElement;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;
use crate::StorageData;

/// Query to select all aliases in the database.
///
/// The result will be number of returned aliases and list
/// of elements with a single property `String("alias")` holding
/// the value `String`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectAllAliasesQuery {}

impl Query for SelectAllAliasesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let mut aliases = db.aliases();
        aliases.sort();
        result.elements.reserve(aliases.len());
        result.result = aliases.len() as i64;

        for alias in aliases {
            result.elements.push(DbElement {
                id: alias.1,
                from: None,
                to: None,
                values: vec![("alias", alias.0).into()],
            });
        }

        Ok(result)
    }
}

impl Query for &SelectAllAliasesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}
