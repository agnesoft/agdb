use crate::DbElement;
use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbValue;
use crate::Query;
use crate::QueryCondition;
use crate::QueryConditionData;
use crate::QueryId;
use crate::QueryResult;
use crate::StorageData;
use crate::Transaction;
use crate::db::db_key_order::DbKeyOrder;
use crate::query_builder::search::SearchQueryBuilder;
use std::cmp::Ordering;

/// Search algorithm to be used
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchQueryAlgorithm {
    /// Examines each distance level from the search origin in full
    /// before continuing with the next level. E.g. when starting at
    /// a node it first examines all the edges and then nodes they lead
    /// to.
    BreadthFirst,

    /// Examines maximum distance it can reach following every element.
    /// E.g. when starting at a node it will go `edge -> node -> edge -> node`
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
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
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

    /// When true and both origin and destination are set,
    /// the path search follows edges in reverse (incoming edges).
    /// Set by the `to().from()` builder order.
    pub reverse: bool,
}

impl Query for SearchQuery {
    fn process<Store: StorageData>(&self, db: &Transaction<Store>) -> Result<QueryResult, DbError> {
        let mut result = QueryResult::default();

        for id in self.search(db)? {
            result.elements.push(DbElement {
                id,
                from: db.from_id(id)?,
                to: db.to_id(id)?,
                values: vec![],
            });
        }

        result.result = result.elements.len() as u64;

        Ok(result)
    }
}

impl SearchQuery {
    pub(crate) fn search<Store: StorageData>(
        &self,
        db: &Transaction<Store>,
    ) -> Result<Vec<DbId>, DbError> {
        if self.algorithm == SearchQueryAlgorithm::Index {
            let condition = self.conditions.first().ok_or_else(|| {
                DbError::query(
                    DbErrorType::NotEnoughData,
                    "Index condition is required for index search",
                )
            })?;

            if let QueryConditionData::KeyValue(kvc) = &condition.data {
                let mut ids = db.search_index(&kvc.key, kvc.value.value())?;

                if self.conditions.len() > 1 {
                    let remaining = &self.conditions[1..];
                    let mut filtered = Vec::new();

                    for id in ids {
                        if db.evaluate_conditions(id, 0, remaining)?.is_true() {
                            filtered.push(id);
                        }
                    }

                    ids = filtered;
                }

                if !self.order_by.is_empty() {
                    self.sort(&mut ids, db)?;
                }

                return Ok(self.slice(ids));
            } else {
                return Err(DbError::query(
                    DbErrorType::NotAllowed,
                    "Index condition must be key value",
                ));
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
                Ok(self.slice(ids))
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
                Ok(self.slice(ids))
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
                Ok(self.slice(ids))
            }
        } else {
            let origin = db.db_id(&self.origin)?;
            let destination = db.db_id(&self.destination)?;
            let mut ids = if self.reverse {
                db.search_to_from(origin, destination, &self.conditions)?
            } else {
                db.search_from_to(origin, destination, &self.conditions)?
            };
            self.sort(&mut ids, db)?;
            Ok(self.slice(ids))
        }
    }

    fn sort<Store: StorageData>(
        &self,
        ids: &mut Vec<DbId>,
        db: &Transaction<Store>,
    ) -> Result<(), DbError> {
        let keys = self
            .order_by
            .iter()
            .map(|key_order| match key_order {
                DbKeyOrder::Asc(key) | DbKeyOrder::Desc(key) => key.clone(),
            })
            .collect::<Vec<DbValue>>();

        let key_ids = db.key_ids(&keys)?;
        let mut values = Vec::with_capacity(ids.len() * keys.len());

        for id in ids.iter() {
            let kvs = db.values_by_keys(*id, &key_ids)?;

            for key in &keys {
                values.push(
                    kvs.iter()
                        .find(|kv| kv.key == *key)
                        .map(|kv| kv.value.clone()),
                );
            }
        }

        let row = |pos: usize| &values[pos * keys.len()..(pos + 1) * keys.len()];
        let compare = |left: &usize, right: &usize| {
            for (key_order, (l, r)) in self.order_by.iter().zip(row(*left).iter().zip(row(*right)))
            {
                let ordering = match (l, r) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater,
                    (Some(_), None) => Ordering::Less,
                    (Some(l), Some(r)) => match key_order {
                        DbKeyOrder::Asc(_) => l.cmp(r),
                        DbKeyOrder::Desc(_) => l.cmp(r).reverse(),
                    },
                };

                if ordering != Ordering::Equal {
                    return ordering;
                }
            }

            left.cmp(right)
        };

        let mut order = (0..ids.len()).collect::<Vec<usize>>();
        let end = if self.limit == 0 {
            order.len()
        } else {
            usize::try_from(self.offset.saturating_add(self.limit)).unwrap_or(usize::MAX)
        };

        if end < order.len() {
            order.select_nth_unstable_by(end, compare);
            order.truncate(end);
        }

        order.sort_unstable_by(compare);
        *ids = order.into_iter().map(|pos| ids[pos]).collect();

        Ok(())
    }

    fn slice(&self, mut ids: Vec<DbId>) -> Vec<DbId> {
        let offset = usize::try_from(self.offset).unwrap_or(usize::MAX);
        ids.drain(..offset.min(ids.len()));

        if self.limit != 0 {
            ids.truncate(usize::try_from(self.limit).unwrap_or(usize::MAX));
        }

        ids
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
            reverse: false,
        }
    }
}

impl Query for &SearchQuery {
    fn process<Store: StorageData>(&self, db: &Transaction<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}

impl SearchQueryBuilder for SearchQuery {
    fn search_mut(&mut self) -> &mut SearchQuery {
        self
    }
}
