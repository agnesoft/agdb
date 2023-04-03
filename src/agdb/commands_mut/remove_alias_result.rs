use super::remove_alias::RemoveAlias;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveAliasResult(pub(crate) RemoveAlias);

impl RemoveAliasResult {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        let undo = self.0.process(db, context)?;

        if undo != CommandsMut::None {
            result.result -= 1;
        }

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
            RemoveAliasResult(RemoveAlias {
                alias: String::new()
            })
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAliasResult(RemoveAlias {
                alias: String::new()
            }),
            RemoveAliasResult(RemoveAlias {
                alias: String::new()
            })
        );
    }
}
