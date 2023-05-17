use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use super::QueryMut;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::DbKeyValue;
use crate::QueryError;
use crate::QueryResult;

pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub values: QueryValues,
    pub each: bool,
}

impl QueryMut for InsertEdgesQuery {
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        if let QueryIds::Ids(from) = &self.from {
            if let QueryIds::Ids(to) = &self.to {
                let ids = if self.each || from.len() != to.len() {
                    self.many_to_many_each(db, from, to)?
                } else {
                    self.many_to_many(db, from, to)?
                };
                result.result = ids.len() as i64;
                result.elements = ids
                    .into_iter()
                    .map(|id| DbElement {
                        index: id,
                        values: vec![],
                    })
                    .collect();

                return Ok(());
            }
        }

        Err(QueryError::from("Invalid insert edges query"))
    }
}

impl InsertEdgesQuery {
    fn many_to_many(
        &self,
        db: &mut Db,
        from: &[QueryId],
        to: &[QueryId],
    ) -> Result<Vec<DbId>, QueryError> {
        let mut ids = vec![];
        ids.reserve(from.len());
        let values = self.values(from.len())?;

        for ((from, to), key_values) in from.iter().zip(to).zip(values) {
            let db_id = db.insert_edge(from, to)?;
            ids.push(db_id);

            for key_value in key_values {
                db.insert_key_value(db_id, &key_value.key, &key_value.value)?;
            }
        }

        Ok(ids)
    }

    fn many_to_many_each(
        &self,
        db: &mut Db,
        from: &[QueryId],
        to: &[QueryId],
    ) -> Result<Vec<DbId>, QueryError> {
        let count = from.len() * to.len();
        let mut ids = vec![];
        ids.reserve(count);
        let values = self.values(count)?;
        let mut index = 0;

        for from in from {
            for to in to {
                let db_id = db.insert_edge(from, to)?;
                ids.push(db_id);

                for key_value in values[index] {
                    db.insert_key_value(db_id, &key_value.key, &key_value.value)?;
                }

                index += 1;
            }
        }

        Ok(ids)
    }

    fn values(&self, count: usize) -> Result<Vec<&Vec<DbKeyValue>>, QueryError> {
        let values = match &self.values {
            QueryValues::Ids(_) => return Err(QueryError::from("Invalid insert query")),
            QueryValues::Single(v) => vec![v; std::cmp::max(1, count)],
            QueryValues::Multi(v) => v.iter().collect(),
        };

        if values.len() != count {
            return Err(QueryError::from(format!(
                "Values len '{}' do not match the insert count '{count}'",
                values.len()
            )));
        }

        Ok(values)
    }
}
