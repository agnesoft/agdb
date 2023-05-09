use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbElement;
use crate::DbKeyValue;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct SelectId {
    pub id: QueryId,
}

impl SelectId {
    pub(crate) fn redo(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let db_id = match &self.id {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            SelectId {
                id: QueryId::from(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            SelectId {
                id: QueryId::from(0)
            },
            SelectId {
                id: QueryId::from(0)
            }
        );
    }
}
