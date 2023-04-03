use super::remove_alias::RemoveAlias;
use super::CommandsMut;
use crate::Db;
use crate::DbId;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertAliasId {
    pub(crate) id: DbId,
    pub(crate) alias: String,
}

impl InsertAliasId {
    pub(crate) fn process(&self, db: &mut Db) -> Result<CommandsMut, QueryError> {
        if self.alias.is_empty() {
            return Err(QueryError::from("Empty alias is not allowed"));
        }

        db.aliases.insert(&self.alias, &self.id)?;

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
            InsertAliasId {
                id: DbId { id: 0 },
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAliasId {
                id: DbId { id: 0 },
                alias: String::new()
            },
            InsertAliasId {
                id: DbId { id: 0 },
                alias: String::new()
            }
        );
    }
}
