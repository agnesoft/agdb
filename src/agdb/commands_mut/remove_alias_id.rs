use super::insert_alias_id::InsertAliasId;
use super::CommandsMut;
use crate::Db;
use crate::DbId;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveAliasId {
    pub(crate) id: DbId,
}

impl RemoveAliasId {
    pub(crate) fn process(&self, db: &mut Db) -> Result<CommandsMut, QueryError> {
        if let Some(alias) = db.aliases.key(&self.id)? {
            db.aliases.remove_value(&self.id)?;
            Ok(CommandsMut::InsertAliasId(InsertAliasId {
                id: self.id,
                alias,
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
        format!("{:?}", RemoveAliasId { id: DbId { id: 0 } });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAliasId { id: DbId { id: 0 } },
            RemoveAliasId { id: DbId { id: 0 } }
        );
    }
}
