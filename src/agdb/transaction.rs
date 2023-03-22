use crate::commands::select_id::SelectId;
use crate::commands::Commands;
use crate::db::db_data;
use crate::db::db_data::DbData;
use crate::query::Query;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

pub struct Transaction<'a> {
    pub(crate) data: &'a DbData,
}

impl<'a> Transaction<'a> {
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

    pub(crate) fn commit(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub(crate) fn rollback(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
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
        let index = db_data::index_from_id(&data.id, self.data)?;
        result.result += 1;
        result.elements.push(DbElement {
            index,
            values: vec![],
        });
        Ok(())
    }
}
