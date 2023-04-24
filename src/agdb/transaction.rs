use crate::commands::select_id::SelectId;
use crate::commands::Commands;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

pub struct Transaction<'a> {
    db: &'a Db,
}

impl<'a> Transaction<'a> {
    pub fn new(data: &'a Db) -> Self {
        Self { db: data }
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        for command in query.commands()? {
            self.process_command(command, &mut result)?;
        }

        Ok(result)
    }

    fn process_command(
        &self,
        command: Commands,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        match command {
            Commands::SelectId(data) => self.select_id(data, result),
        }
    }

    fn select_id(&self, data: SelectId, result: &mut QueryResult) -> Result<(), QueryError> {
        let db_id = match data.id {
            QueryId::Id(id) => DbId(id),
            QueryId::Alias(alias) => self
                .db
                .aliases
                .value(&alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        };

        let _ = self
            .db
            .indexes
            .value(&db_id)?
            .ok_or(QueryError::from(format!("Id '{}' not found", db_id.0)))?;

        result.result += 1;
        result.elements.push(DbElement {
            index: db_id,
            values: vec![],
        });

        Ok(())
    }
}
