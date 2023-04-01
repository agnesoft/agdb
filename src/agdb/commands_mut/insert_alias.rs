use super::remove_alias::RemoveAlias;
use super::CommandsMut;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    pub(crate) id: QueryId,
    pub(crate) alias: String,
}

impl InsertAlias {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let index = db.index_from_id(&self.id)?;
        db.aliases.insert(&self.alias, &index)?;
        result.result += 1;
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
                id: QueryId::Id(0),
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias {
                id: QueryId::Id(0),
                alias: String::new()
            },
            InsertAlias {
                id: QueryId::Id(0),
                alias: String::new()
            }
        );
    }
}
