use super::remove_alias::RemoveAlias;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    pub(crate) alias: String,
}

impl InsertAlias {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        db.aliases.insert(&self.alias, &context.id)?;

        Ok(CommandsMut::RemoveAlias(RemoveAlias {
            alias: self.alias.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAlias {
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias {
                alias: String::new()
            },
            InsertAlias {
                alias: String::new()
            }
        );
    }
}
