use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::insert_alias::InsertAlias;
use crate::commands_mut::CommandsMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

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

    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.ids {
            QueryIds::Ids(ids) => {
                if ids.len() != self.aliases.len() {
                    return Err(QueryError::from(
                        "Ids and aliases must have the same length",
                    ));
                }

                for (id, alias) in ids.iter().zip(&self.aliases) {
                    if alias.is_empty() {
                        return Err(QueryError::from("Empty alias is not allowed"));
                    }

                    let db_id = db.db_id(id)?;
                    db.insert_alias(db_id, alias)?;
                    result.result += 1;
                }

                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid insert aliases query")),
        }
    }
}

impl InsertAliasesQuery {
    fn ids(&self, ids: &[QueryId]) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for (id, alias) in ids.iter().zip(&self.aliases) {
            commands.push(CommandsMut::InsertAlias(InsertAlias::new(
                id.clone(),
                alias.to_string(),
            )));
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
                QueryId::Id(DbId(0)),
                "alias".to_string(),
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
                QueryId::Id(DbId(0)),
                "alias".to_string(),
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
