use super::query_condition::QueryCondition;
use super::query_id::QueryId;
use crate::db::db_key::DbKeyOrder;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::DbKey;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;
use std::cmp::Ordering;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    pub origin: QueryId,
    pub destination: QueryId,
    pub limit: u64,
    pub offset: u64,
    pub order_by: Vec<DbKeyOrder>,
    pub conditions: Vec<QueryCondition>,
}

impl Query for SearchQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        for id in self.search(db)? {
            result.elements.push(DbElement { id, values: vec![] });
        }

        result.result = result.elements.len() as i64;

        Ok(())
    }
}

impl SearchQuery {
    pub(crate) fn search(&self, db: &Db) -> Result<Vec<DbId>, QueryError> {
        if self.destination == QueryId::Id(DbId(0)) {
            let origin = db.db_id(&self.origin)?;

            if self.order_by.is_empty() {
                db.search_from(origin, self.limit, self.offset, &self.conditions)
            } else {
                let mut ids = db.search_from(origin, 0, 0, &self.conditions)?;
                self.sort(&mut ids, db)?;
                self.slice(ids)
            }
        } else if self.origin == QueryId::Id(DbId(0)) {
            let destination = db.db_id(&self.destination)?;

            if self.order_by.is_empty() {
                db.search_to(destination, self.limit, self.offset, &self.conditions)
            } else {
                let mut ids = db.search_to(destination, 0, 0, &self.conditions)?;
                self.sort(&mut ids, db)?;
                self.slice(ids)
            }
        } else {
            let origin = db.db_id(&self.origin)?;
            let destination = db.db_id(&self.destination)?;
            let mut ids = db.search_from_to(origin, destination, &self.conditions)?;
            self.sort(&mut ids, db)?;
            self.slice(ids)
        }
    }

    fn sort(&self, ids: &mut [DbId], db: &Db) -> Result<(), QueryError> {
        let keys = self
            .order_by
            .iter()
            .map(|key_order| match key_order {
                DbKeyOrder::Asc(key) | DbKeyOrder::Desc(key) => key.clone(),
            })
            .collect::<Vec<DbKey>>();

        ids.sort_by(|left, right| {
            let left_values = db.values_by_keys(*left, &keys).unwrap_or_default();
            let right_values = db.values_by_keys(*right, &keys).unwrap_or_default();

            for (key_order, key) in self.order_by.iter().zip(&keys) {
                let left_kv = left_values.iter().find(|kv| kv.key == *key);
                let right_kv = right_values.iter().find(|kv| kv.key == *key);

                let ordering = match (left_kv, right_kv) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(_), None) => Ordering::Less,
                    (Some(l), Some(r)) => match key_order {
                        DbKeyOrder::Asc(_) => l.value.cmp(&r.value),
                        DbKeyOrder::Desc(_) => l.value.cmp(&r.value).reverse(),
                    },
                };

                if ordering != Ordering::Equal {
                    return ordering;
                }
            }

            Ordering::Equal
        });

        Ok(())
    }

    fn slice(&self, mut ids: Vec<DbId>) -> Result<Vec<DbId>, QueryError> {
        Ok(match (self.limit, self.offset) {
            (0, 0) => ids,
            (0, _) => ids[self.offset as usize..].to_vec(),
            (_, 0) => {
                ids.truncate(self.limit as usize);
                ids
            }
            (_, _) => ids[self.offset as usize..(self.offset + self.limit) as usize].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            }
        );
    }

    #[test]
    fn derived_from_clone() {
        let left = SearchQuery {
            origin: QueryId::from(0),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        };
        let right = left.clone();
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            },
            SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            }
        );
    }
}
