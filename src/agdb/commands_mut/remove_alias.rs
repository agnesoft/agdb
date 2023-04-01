use super::insert_alias::InsertAlias;
use super::CommandsMut;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveAlias {
    pub(crate) alias: String,
}

impl RemoveAlias {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let id = db.aliases.value(&self.alias)?.unwrap_or_default();
        db.aliases.remove_key(&self.alias)?;
        result.result += 1;
        Ok(CommandsMut::InsertAlias(InsertAlias {
            id: QueryId::Id(id),
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
