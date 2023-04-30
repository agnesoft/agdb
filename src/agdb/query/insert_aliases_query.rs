use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::insert_alias::InsertAlias;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::CommandsMut;
use crate::QueryError;

pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl QueryMut for InsertAliasesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        match &self.ids {
            QueryIds::Ids(ids) => Ok(self.ids(ids)),
            QueryIds::Search(_) => Err(QueryError::from("Invalid insert aliases query")),
        }
    }
}

impl InsertAliasesQuery {
    fn id(&self, id: &QueryId, new_alias: &str) -> Vec<CommandsMut> {
        match id {
            QueryId::Id(id) => {
                vec![CommandsMut::InsertAlias(InsertAlias::new(
                    new_alias.to_string(),
                    Some(*id),
                ))]
            }
            QueryId::Alias(alias) => {
                vec![
                    CommandsMut::RemoveAlias(RemoveAlias {
                        id: None,
                        alias: alias.clone(),
                        result: false,
                    }),
                    CommandsMut::InsertAlias(InsertAlias::new(new_alias.to_string(), None)),
                ]
            }
        }
    }

    fn ids(&self, ids: &[QueryId]) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for (id, alias) in ids.iter().zip(&self.aliases) {
            commands.extend(self.id(id, alias));
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::search_query::SearchQuery;
    use crate::DbId;

    #[test]
    fn valid_id() {
        let query = InsertAliasesQuery {
            ids: QueryIds::Ids(vec![QueryId::from(0)]),
            aliases: vec!["alias".to_string()],
        };

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::InsertAlias(InsertAlias::new(
                "alias".to_string(),
                Some(DbId(0))
            ))])
        )
    }

    #[test]
    fn valid_ids() {
        let query = InsertAliasesQuery {
            ids: QueryIds::Ids(vec![QueryId::from(0)]),
            aliases: vec!["alias".to_string()],
        };

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::InsertAlias(InsertAlias::new(
                "alias".to_string(),
                Some(DbId(0))
            ))])
        )
    }

    #[test]
    fn invalid_query_all() {
        let query = InsertAliasesQuery {
            ids: QueryIds::Search(SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            aliases: vec![],
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert aliases query").description
        );
    }
}
