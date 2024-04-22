use crate::DbElement;
use crate::Query;
use crate::QueryIds;
use crate::QueryResult;

/// Query to select number of edges of given node ids.
/// All of the ids must exist in the database. If any
/// of the ids is not a node the result will be 0 (not
/// an error).
///
/// The result will be number of elements returned and the list
/// of elements with a single property `String("edge_count")` with
/// a value `u64`.
///
/// NOTE: Self-referential edges are counted twice as if they
/// were coming from another edge. Therefore the edge count
/// might be greater than number of unique db elements.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, PartialEq)]
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
    fn process<Store: crate::StorageData>(
        &self,
        db: &crate::DbImpl<Store>,
    ) -> Result<crate::QueryResult, crate::QueryError> {
        let mut result = QueryResult::default();

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
        result.result = db_ids.len() as i64;

        for id in db_ids {
            result.elements.push(DbElement {
                id,
                from: db.from_id(id),
                to: db.to_id(id),
                values: vec![("edge_count", db.edge_count(id, self.from, self.to)?).into()],
            });
        }

        Ok(result)
    }
}
