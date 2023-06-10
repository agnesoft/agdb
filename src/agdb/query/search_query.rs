use super::query_condition::QueryCondition;
use super::query_id::QueryId;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::DbKey;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    pub origin: QueryId,
    pub destination: QueryId,
    pub limit: u64,
    pub offset: u64,
    pub order_by: Vec<DbKey>,
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
        let ids = if self.destination == QueryId::Id(DbId(0)) {
            let origin = db.db_id(&self.origin)?;
            db.search_from(origin, self.limit, self.offset)?
        } else if self.origin == QueryId::Id(DbId(0)) {
            let destination = db.db_id(&self.destination)?;
            db.search_to(destination, self.limit, self.offset)?
        } else {
            let origin = db.db_id(&self.origin)?;
            let destination = db.db_id(&self.destination)?;
            let mut path = db.search_from_to(origin, destination)?;

            match (self.limit, self.offset) {
                (0, 0) => path,
                (0, _) => path[self.offset as usize..].to_vec(),
                (_, 0) => {
                    path.truncate(self.limit as usize);
                    path
                }
                (_, _) => path[self.offset as usize..(self.offset + self.limit) as usize].to_vec(),
            }
        };

        //order result by self.order_by

        Ok(ids)
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
