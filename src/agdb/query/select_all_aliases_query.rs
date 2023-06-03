use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectAllAliases {}

impl Query for SelectAllAliases {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let mut aliases = db.aliases();
        aliases.sort();
        result.elements.reserve(aliases.len());
        result.result = aliases.len() as i64;

        for alias in aliases {
            result.elements.push(DbElement {
                index: alias.1,
                values: vec![("alias", alias.0).into()],
            });
        }

        Ok(())
    }
}
