use crate::DbElement;
use crate::DbError;
use crate::DbImpl;
use crate::Query;
use crate::QueryIds;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;
use crate::query_builder::search::SearchQueryBuilder;

/// Query to select number of edges of given node ids.
/// All of the ids must exist in the database.
///
/// The result is the sum of all selected edge counts.
/// The elements still contain individual
/// edge counts in property `String("edge_count")` as `u64`.
/// If any of the element ids are edges their count will be 0.
///
/// NOTE: Self-referential edges are counted twice as if they
/// were coming from another edge. Therefore the edge count
/// might be greater than number of unique db elements.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(SearchQueryBuilder))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectEdgeCountQuery {
    /// Ids of the nodes to select edge count for.
    pub ids: QueryIds,

    /// If set to `true` the query will count outgoing edges
    /// from the nodes.
    pub from: bool,

    /// If set to `true` the query will count incoming edges
    /// to the nodes.
    pub to: bool,
}

impl Query for SelectEdgeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        let mut result = QueryResult::default();
        let mut total_count = 0_u64;

        let db_ids = match &self.ids {
            QueryIds::Ids(ids) => {
                let mut db_ids = Vec::with_capacity(ids.len());

                for query_id in ids {
                    db_ids.push(db.db_id(query_id)?);
                }

                db_ids
            }
            QueryIds::Search(search_query) => search_query.search(db)?,
        };

        result.elements.reserve(db_ids.len());

        for id in db_ids {
            let edge_count = db.edge_count(id, self.from, self.to)?;
            total_count += edge_count;
            result.elements.push(DbElement {
                id,
                from: db.from_id(id)?,
                to: db.to_id(id)?,
                values: vec![("edge_count", edge_count).into()],
            });
        }

        result.result = total_count;

        Ok(result)
    }
}

impl Query for &SelectEdgeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SearchQueryBuilder for SelectEdgeCountQuery {
    fn search_mut(&mut self) -> &mut SearchQuery {
        if let QueryIds::Search(search) = &mut self.ids {
            search
        } else {
            panic!("Expected search query");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn missing_search() {
        SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![]),
            from: false,
            to: false,
        }
        .search_mut();
    }
}
