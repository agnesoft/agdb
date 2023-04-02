use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::insert_alias::InsertAlias;
use crate::commands_mut::insert_alias_id::InsertAliasId;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::CommandsMut;
use crate::DbId;
use crate::QueryError;

pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl QueryMut for InsertAliasesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        match &self.ids {
            QueryIds::Id(id) => Ok(self.id(id, &self.aliases[0])),
            QueryIds::Ids(ids) => Ok(self.ids(ids)),
            QueryIds::All | QueryIds::Search(_) => {
                Err(QueryError::from("Invalid insert aliases query"))
            }
        }
    }
}

impl InsertAliasesQuery {
    fn id(&self, id: &QueryId, new_alias: &str) -> Vec<CommandsMut> {
        match id {
            QueryId::Id(id) => {
                vec![CommandsMut::InsertAliasId(InsertAliasId {
                    id: DbId { id: *id },
                    alias: new_alias.to_string(),
                })]
            }
            QueryId::Alias(alias) => {
                vec![
                    CommandsMut::RemoveAlias(RemoveAlias {
                        alias: alias.clone(),
                    }),
                    CommandsMut::InsertAlias(InsertAlias {
                        alias: new_alias.to_string(),
                    }),
                ]
            }
        }
    }

    fn ids(&self, ids: &[QueryId]) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for (id, alias) in ids.iter().zip(&self.aliases) {
            commands.extend(self.id(id, &alias));
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
            Ok(vec![CommandsMut::InsertAliasId(InsertAliasId {
                id: DbId { id: 0 },
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
            Ok(vec![CommandsMut::InsertAliasId(InsertAliasId {
                id: DbId { id: 0 },
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
