use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::Query;
use crate::Db;
use crate::DbElement;
use crate::DbKeyValue;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectQuery(pub QueryIds);

impl Query for SelectQuery {
    fn redo(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                for id in ids {
                    Self::select_id(id, db, result)?
                }
                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select query")),
        }
    }
}

impl SelectQuery {
    fn select_id(id: &QueryId, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let db_id = match id {
            QueryId::Id(id) => {
                let _ = db
                    .indexes
                    .value(id)?
                    .ok_or(QueryError::from(format!("Id '{}' not found", id.0)))?;
                *id
            }
            QueryId::Alias(alias) => db
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        };

        let mut element = DbElement {
            index: db_id,
            values: vec![],
        };

        for key_value_index in db.values.values(&db_id)? {
            let key = db.value(&key_value_index.key)?;
            let value = db.value(&key_value_index.value)?;

            element.values.push(DbKeyValue { key, value });
        }

        result.result += 1;
        result.elements.push(element);

        Ok(())
    }
}
