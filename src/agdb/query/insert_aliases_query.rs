use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::insert_alias::InsertAlias;
use crate::commands_mut::CommandsMut;
use crate::QueryError;

pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl QueryMut for InsertAliasesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        match &self.ids {
            QueryIds::Id(id) => Ok(self.id(id)),
            QueryIds::Ids(ids) => Ok(self.ids(ids)),
            QueryIds::All | QueryIds::Search(_) => {
                Err(QueryError::from("Invalid insert aliases query"))
            }
        }
    }
}

impl InsertAliasesQuery {
    fn id(&self, id: &QueryId) -> Vec<CommandsMut> {
        vec![CommandsMut::InsertAlias(InsertAlias {
            id: id.clone(),
            alias: self.aliases[0].clone(),
        })]
    }

    fn ids(&self, ids: &[QueryId]) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for (id, alias) in ids.iter().zip(self.aliases.iter()) {
            commands.push(CommandsMut::InsertAlias(InsertAlias {
                id: id.clone(),
                alias: alias.clone(),
            }));
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_id() {
        let query = InsertAliasesQuery {
            ids: QueryIds::Id(QueryId::Id(0)),
            aliases: vec!["alias".to_string()],
        };

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::InsertAlias(InsertAlias {
                id: QueryId::Id(0),
                alias: "alias".to_string()
            })])
        )
    }

    #[test]
    fn valid_ids() {
        let query = InsertAliasesQuery {
            ids: QueryIds::Ids(vec![QueryId::Id(0)]),
            aliases: vec!["alias".to_string()],
        };

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::InsertAlias(InsertAlias {
                id: QueryId::Id(0),
                alias: "alias".to_string()
            })])
        )
    }

    #[test]
    fn invalid_query_all() {
        let query = InsertAliasesQuery {
            ids: QueryIds::All,
            aliases: vec![],
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert aliases query").description
        );
    }
}
