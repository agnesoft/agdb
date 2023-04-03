use super::insert_alias_id::InsertAliasId;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveAlias {
    pub(crate) alias: String,
}

impl RemoveAlias {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        if let Some(id) = db.aliases.value(&self.alias)? {
            context.id = id;
            db.aliases.remove_key(&self.alias)?;
            Ok(CommandsMut::InsertAliasId(InsertAliasId {
                id: context.id,
                alias: self.alias.clone(),
            }))
        } else {
            Ok(CommandsMut::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            RemoveAlias {
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAlias {
                alias: String::new()
            },
            RemoveAlias {
                alias: String::new()
            }
        );
    }
}
