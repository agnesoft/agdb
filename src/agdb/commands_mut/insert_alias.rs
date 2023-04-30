use super::remove_alias::RemoveAlias;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    pub(crate) id: Option<DbId>,
    pub(crate) alias: String,
    pub(crate) result: bool,
}

impl InsertAlias {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        let undo = insert_alias(db, &self.id.unwrap_or(context.id), &self.alias)?;

        if self.result {
            result.result += 1;
        }

        Ok(undo)
    }
}

fn insert_alias(db: &mut Db, id: &DbId, alias: &String) -> Result<CommandsMut, QueryError> {
    if alias.is_empty() {
        return Err(QueryError::from("Empty alias is not allowed"));
    }

    db.aliases.insert(alias, id)?;

    Ok(CommandsMut::RemoveAlias(RemoveAlias {
        id: Some(*id),
        alias: alias.clone(),
        result: false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAlias {
                id: None,
                alias: String::new(),
                result: false
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias {
                id: None,
                alias: String::new(),
                result: false
            },
            InsertAlias {
                id: None,
                alias: String::new(),
                result: false
            }
        );
    }
}
