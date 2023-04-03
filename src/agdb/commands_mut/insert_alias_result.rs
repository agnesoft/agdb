use super::insert_alias::InsertAlias;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertAliasResult(pub(crate) InsertAlias);

impl InsertAliasResult {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        let undo = self.0.process(db, context)?;
        result.result += 1;
        Ok(undo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAliasResult(InsertAlias {
                alias: String::new()
            })
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAliasResult(InsertAlias {
                alias: String::new()
            }),
            InsertAliasResult(InsertAlias {
                alias: String::new()
            })
        );
    }
}
