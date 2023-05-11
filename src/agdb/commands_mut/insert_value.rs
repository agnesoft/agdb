use crate::collections::dictionary::dictionary_index::DictionaryIndex;
use crate::db::db_key_value_index::DbKeyValueIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbId;
use crate::DbKeyValue;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertValue {
    id: QueryId,
    db_id: DbId,
    values: Vec<DbKeyValue>,
    indexes: Vec<DbKeyValueIndex>,
}

impl InsertValue {
    pub(crate) fn new(id: QueryId, values: Vec<DbKeyValue>) -> Self {
        Self {
            id,
            db_id: DbId(0),
            values,
            indexes: vec![],
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        self.db_id = match &self.id {
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
                .ok_or(QueryError::from(format!("Alias '{}' not found", alias)))?,
        };

        for key_value in &self.values {
            let key = db.insert_value(&key_value.key)?;
            let value = db.insert_value(&key_value.value)?;
            let index = DbKeyValueIndex { key, value };
            db.values.insert(&self.db_id, &index)?;
            self.indexes.push(index);
            result.result += 1;
        }

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        for index in self.indexes {
            db.values.remove_value(&self.db_id, &index)?;

            if index.key.is_value() {
                db.dictionary.remove(DictionaryIndex(index.key.index()))?;
            }

            if index.value.is_value() {
                db.dictionary.remove(DictionaryIndex(index.value.index()))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertValue::new(QueryId::Id(DbId(0)), vec![]),
            InsertValue::new(QueryId::Id(DbId(0)), vec![])
        );
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertValue::new(QueryId::Id(DbId(0)), vec![]));
    }
}
