use super::insert_alias_id::InsertAliasId;
use super::CommandsMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertAliasIdResult(pub(crate) InsertAliasId);

impl InsertAliasIdResult {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let undo = self.0.process(db)?;
        result.result += 1;
        Ok(undo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbId;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAliasIdResult(InsertAliasId {
                id: DbId { id: 0 },
                alias: String::new()
            })
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAliasIdResult(InsertAliasId {
                id: DbId { id: 0 },
                alias: String::new()
            }),
            InsertAliasIdResult(InsertAliasId {
                id: DbId { id: 0 },
                alias: String::new()
            })
        );
    }
}
