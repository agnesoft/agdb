use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use super::QueryMut;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::DbKeyValue;
use crate::QueryError;
use crate::QueryResult;

/// Query to inserts edges to the database. The `from`
/// and `to` ids must exist in the database. There must be
/// enough `values` for all new edges unless set to `Single`
/// in which case they will be uniformly applied to all new
/// edges. The `each` flag is only useful if `from and `to` are
/// symmetric (same length) but you still want to connect every
/// origin to every destination. By default it would connect only
/// the pairs. For asymmetric inserts `each` is assumed.
///
/// The result will contain number of edges inserted and elements with
/// their ids but no properties.
pub struct InsertEdgesQuery {
    /// Origins
    pub from: QueryIds,

    /// Destinations
    pub to: QueryIds,

    /// Key value pairs to be associated with
    /// the new edges.
    pub values: QueryValues,

    /// If `true` create an edge between each origin
    /// and destination.
    pub each: bool,
}

impl QueryMut for InsertEdgesQuery {
    fn process(&self, db: &mut Db) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let from = Self::db_ids(&self.from, db)?;
        let to: Vec<DbId> = Self::db_ids(&self.to, db)?;

        let ids = if self.each || from.len() != to.len() {
            self.many_to_many_each(db, &from, &to)?
        } else {
            self.many_to_many(db, &from, &to)?
        };

        result.result = ids.len() as i64;
        result.elements = ids
            .into_iter()
            .map(|id| DbElement { id, values: vec![] })
            .collect();

        Ok(result)
    }
}

impl InsertEdgesQuery {
    fn db_ids(query_ids: &QueryIds, db: &Db) -> Result<Vec<DbId>, QueryError> {
        Ok(match &query_ids {
            QueryIds::Ids(query_ids) => {
                let mut ids = vec![];
                ids.reserve(query_ids.len());

                for query_id in query_ids {
                    ids.push(db.db_id(query_id)?);
                }

                ids
            }
            QueryIds::Search(search_query) => search_query
                .search(db)?
                .into_iter()
                .filter(|id| id.0 > 0)
                .collect(),
        })
    }

    fn many_to_many(
        &self,
        db: &mut Db,
        from: &[DbId],
        to: &[DbId],
    ) -> Result<Vec<DbId>, QueryError> {
        let mut ids = vec![];
        ids.reserve(from.len());
        let values = self.values(from.len())?;

        for ((from, to), key_values) in from.iter().zip(to).zip(values) {
            let db_id = db.insert_edge(*from, *to)?;
            ids.push(db_id);

            for key_value in key_values {
                db.insert_key_value(db_id, key_value)?;
            }
        }

        Ok(ids)
    }

    fn many_to_many_each(
        &self,
        db: &mut Db,
        from: &[DbId],
        to: &[DbId],
    ) -> Result<Vec<DbId>, QueryError> {
        let count = from.len() * to.len();
        let mut ids = vec![];
        ids.reserve(count);
        let values = self.values(count)?;
        let mut index = 0;

        for from in from {
            for to in to {
                let db_id = db.insert_edge(*from, *to)?;
                ids.push(db_id);

                for key_value in values[index] {
                    db.insert_key_value(db_id, key_value)?;
                }

                index += 1;
            }
        }

        Ok(ids)
    }

    fn values(&self, count: usize) -> Result<Vec<&Vec<DbKeyValue>>, QueryError> {
        let values = match &self.values {
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
