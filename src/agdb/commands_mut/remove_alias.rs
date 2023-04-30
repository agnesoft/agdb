use super::insert_alias::InsertAlias;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveAlias {
    pub(crate) id: Option<DbId>,
    pub(crate) alias: String,
    pub(crate) result: bool,
}

impl RemoveAlias {
    pub(crate) fn redo(
        &mut self,
        db: &mut Db,
        result: &QueryResult,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        todo!()
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        todo!()
    }

    // pub(crate) fn process(
    //     &self,
    //     db: &mut Db,
    //     result: &mut QueryResult,
    //     context: &mut Context,
    // ) -> Result<CommandsMut, QueryError> {
    //     if let Some(id) = &self.id {
    //         if let Some(alias) = db.aliases.key(id)? {
    //             db.aliases.remove_value(id)?;

    //             return Ok(CommandsMut::InsertAlias(InsertAlias {
    //                 id: Some(*id),
    //                 alias,
    //                 result: false,
    //             }));
    //         }
    //     } else if let Some(id) = db.aliases.value(&self.alias)? {
    //         context.id = id;
    //         db.aliases.remove_key(&self.alias)?;

    //         if self.result {
    //             result.result -= 1;
    //         }

    //         return Ok(CommandsMut::InsertAlias(InsertAlias {
    //             id: Some(context.id),
    //             alias: self.alias.clone(),
    //             result: false,
    //         }));
    //     }

    //     Ok(CommandsMut::None)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            RemoveAlias {
                id: None,
                alias: String::new(),
                result: false,
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAlias {
                id: None,
                alias: String::new(),
                result: false,
            },
            RemoveAlias {
                id: None,
                alias: String::new(),
                result: false,
            }
        );
    }
}
