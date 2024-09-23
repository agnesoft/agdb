use crate::db::db_key_order::DbKeyOrder;
use crate::query_builder::search::SearchQueryBuilder;
use crate::DbElement;
use crate::DbId;
use crate::DbImpl;
use crate::DbValue;
use crate::Query;
use crate::QueryCondition;
use crate::QueryConditionData;
use crate::QueryError;
use crate::QueryId;
use crate::QueryResult;
use crate::StorageData;
use std::cmp::Ordering;

/// Search algorithm to be used
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchQueryAlgorithm {
    /// Examines each distance level from the search origin in full
    /// before continuing with the next level. E.g. when starting at
    /// a node it first examines all the edges and then nodes they lead
    /// to.
    BreadthFirst,

    /// Examines maximum distance it can reach following every element.
    /// E.g. when starting at anode it will go `edge -> node -> edge -> node`
    /// until it reaches dead end or encounters already visited element.
    DepthFirst,

    /// Bypasses the graph traversal and inspects only the index specified
    /// as the first condition (key).
    Index,

    /// Examines all elements in the database disregarding the graph structure
    /// or any relationship between the elements.
    Elements,
}

/// Query to search for ids in the database following the graph.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SearchQuery {
    /// Search algorithm to be used. Will be bypassed for path
    /// searches that unconditionally use A*.
    pub algorithm: SearchQueryAlgorithm,

    /// Starting element of the search.
    pub origin: QueryId,

    /// Target element of the path search (if origin is specified)
    /// or starting element of the reverse search (if origin is not specified).
    pub destination: QueryId,

    /// How many elements maximum to return.
    pub limit: u64,

    /// How many elements that would be returned should be
    /// skipped in the result.
    pub offset: u64,

    /// Order of the elements in the result. The sorting happens before
    /// `offset` and `limit` are applied.
    pub order_by: Vec<DbKeyOrder>,

    /// Set of conditions every element must satisfy to be included in the
    /// result. Some conditions also influence the search path as well.
    pub conditions: Vec<QueryCondition>,
}

impl Query for SearchQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        for id in self.search(db)? {
            result.elements.push(DbElement {
                id,
                from: db.from_id(id),
                to: db.to_id(id),
                values: vec![],
            });
        }

        result.result = result.elements.len() as i64;

        Ok(result)
    }
}

impl SearchQuery {
    pub(crate) fn search<Store: StorageData>(
        &self,
        db: &DbImpl<Store>,
    ) -> Result<Vec<DbId>, QueryError> {
        if self.algorithm == SearchQueryAlgorithm::Index {
            let condition = self.conditions.first().ok_or("Index condition missing")?;

            if let QueryConditionData::KeyValue { key, value } = &condition.data {
                let ids = db.search_index(key, value.value())?;
                return Ok(ids);
            } else {
                return Err("Index condition must be key value".into());
            }
        }

        if self.algorithm == SearchQueryAlgorithm::Elements {
            if self.order_by.is_empty() {
                db.search_from(
                    DbId(0),
                    self.algorithm,
                    self.limit,
                    self.offset,
                    &self.conditions,
                )
            } else {
                let mut ids = db.search_from(DbId(0), self.algorithm, 0, 0, &self.conditions)?;
                self.sort(&mut ids, db)?;
                self.slice(ids)
            }
        } else if self.destination == QueryId::Id(DbId(0)) {
            let origin = db.db_id(&self.origin)?;

            if self.order_by.is_empty() {
                db.search_from(
                    origin,
                    self.algorithm,
                    self.limit,
                    self.offset,
                    &self.conditions,
                )
            } else {
                let mut ids = db.search_from(origin, self.algorithm, 0, 0, &self.conditions)?;
                self.sort(&mut ids, db)?;
                self.slice(ids)
            }
        } else if self.origin == QueryId::Id(DbId(0)) {
            let destination = db.db_id(&self.destination)?;

            if self.order_by.is_empty() {
                db.search_to(
                    destination,
                    self.algorithm,
                    self.limit,
                    self.offset,
                    &self.conditions,
                )
            } else {
                let mut ids = db.search_to(destination, self.algorithm, 0, 0, &self.conditions)?;
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

    fn sort<Store: StorageData>(
        &self,
        ids: &mut [DbId],
        db: &DbImpl<Store>,
    ) -> Result<(), QueryError> {
        let keys = self
            .order_by
            .iter()
            .map(|key_order| match key_order {
                DbKeyOrder::Asc(key) | DbKeyOrder::Desc(key) => key.clone(),
            })
            .collect::<Vec<DbValue>>();

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

    pub(crate) fn new() -> Self {
        Self {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        }
    }
}

impl Query for &SearchQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}

impl SearchQueryBuilder for SearchQuery {
    fn search_mut(&mut self) -> &mut SearchQuery {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let _ = format!(
            "{:?}",
            SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
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
    #[allow(clippy::redundant_clone)]
    fn derived_from_clone() {
        let left = SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
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
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            },
            SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
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
