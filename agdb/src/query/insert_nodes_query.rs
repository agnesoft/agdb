use crate::query::query_values::QueryValues;
use crate::DbElement;
use crate::DbImpl;
use crate::QueryError;
use crate::QueryId;
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
/// The result will contain number of nodes inserted and elements with
/// their ids but no properties.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, PartialEq)]
pub struct InsertNodesQuery {
    /// Number of nodes to be inserted.
    pub count: u64,

    /// Key value pairs to be associated with
    /// the new nodes.
    pub values: QueryValues,

    /// Aliases of the new nodes.
    pub aliases: Vec<String>,
}

impl QueryMut for InsertNodesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();
        let mut ids = vec![];
        let count = std::cmp::max(self.count, self.aliases.len() as u64);
        let values = match &self.values {
            QueryValues::Single(v) => vec![v; std::cmp::max(1, count as usize)],
            QueryValues::Multi(v) => v.iter().collect(),
        };

        if !self.aliases.is_empty() && values.len() != self.aliases.len() {
            return Err(QueryError::from(format!(
                "Values ({}) and aliases ({}) must have the same length",
                values.len(),
                self.aliases.len()
            )));
        }

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
