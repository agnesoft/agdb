use crate::query::query_values::QueryValues;
use crate::DbElement;
use crate::DbId;
use crate::DbImpl;
use crate::QueryError;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::StorageData;

/// Query to insert nodes to the database. Only one of
/// `count`, `values` or `aliases` need to be given as the
/// implementation will derive the count from the other
/// parameters. If `values` is set to `Single` either `count`
/// or `aliases` must be provided however. If `values` are not
/// set to `Single` there must be enough value for `count/aliases`
/// unless they are not se and the count is derived from `values.
///
/// If the `ids` member is empty the query will insert new nodes
/// otherwise it will update the existing nodes. The rules for length
/// of `values` still apply and the search yield or static list must
/// have equal length to the `values` (or the `Single` variant must
/// be used).
///
/// The result will contain number of nodes inserted or updated and elements
/// with their ids but no properties.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct InsertNodesQuery {
    /// Number of nodes to be inserted.
    pub count: u64,

    /// Key value pairs to be associated with
    /// the new nodes.
    pub values: QueryValues,

    /// Aliases of the new nodes.
    pub aliases: Vec<String>,

    /// Optional ids of nodes (optionally a search sub-query).
    /// This can be empty.
    pub ids: QueryIds,
}

impl QueryMut for InsertNodesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();
        let mut ids = vec![];
        let count = std::cmp::max(self.count, self.aliases.len() as u64);
        let query_ids = match &self.ids {
            QueryIds::Ids(ids) => ids
                .iter()
                .map(|query_id| db.db_id(query_id))
                .collect::<Result<Vec<DbId>, QueryError>>()?,
            QueryIds::Search(search_query) => search_query.search(db)?,
        };
        let values = match &self.values {
            QueryValues::Single(v) => vec![v; std::cmp::max(query_ids.len(), count as usize)],
            QueryValues::Multi(v) => v.iter().collect(),
        };

        if values.len() < self.aliases.len() {
            return Err(QueryError::from(format!(
                "Aliases ({}) and values ({}) must have compatible lenghts ({} <= {})",
                self.aliases.len(),
                values.len(),
                self.aliases.len(),
                values.len(),
            )));
        }

        if !query_ids.is_empty() {
            query_ids.iter().try_for_each(|db_id| {
                if db_id.0 < 0 {
                    Err(QueryError::from(format!(
                        "The ids for insert or update must all refer to nodes - edge id '{}' found",
                        db_id.0
                    )))
                } else {
                    Ok(())
                }
            })?;

            if values.len() != query_ids.len() {
                return Err(QueryError::from(format!(
                    "Values ({}) and ids ({}) must have the same length",
                    values.len(),
                    query_ids.len()
                )));
            }

            for ((index, db_id), key_values) in query_ids.iter().enumerate().zip(values) {
                for key_value in key_values {
                    db.insert_or_replace_key_value(*db_id, key_value)?;
                }

                if let Some(alias) = self.aliases.get(index) {
                    db.insert_new_alias(*db_id, alias)?;
                }

                ids.push(*db_id);
            }
        } else {
            for (index, key_values) in values.iter().enumerate() {
                if let Some(alias) = self.aliases.get(index) {
                    if let Ok(db_id) = db.db_id(&QueryId::Alias(alias.to_string())) {
                        ids.push(db_id);

                        for key_value in *key_values {
                            db.insert_or_replace_key_value(db_id, key_value)?;
                        }

                        continue;
                    }
                }

                let db_id = db.insert_node()?;
                ids.push(db_id);

                if let Some(alias) = self.aliases.get(index) {
                    db.insert_new_alias(db_id, alias)?;
                }

                for key_value in *key_values {
                    db.insert_key_value(db_id, key_value)?;
                }
            }
        }

        result.result = ids.len() as i64;
        result.elements = ids
            .into_iter()
            .map(|id| DbElement {
                id,
                from: None,
                to: None,
                values: vec![],
            })
            .collect();

        Ok(result)
    }
}

impl QueryMut for &InsertNodesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}
