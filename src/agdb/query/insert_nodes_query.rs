use super::query_values::QueryValues;
use super::QueryMut;
use crate::Db;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}

impl QueryMut for InsertNodesQuery {
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let mut ids = vec![];
        let count = std::cmp::max(self.count, self.aliases.len() as u64);
        let values = match &self.values {
            QueryValues::Single(v) => vec![v; std::cmp::max(1, count as usize)],
            QueryValues::Multi(v) => v.iter().collect(),
        };

        for (index, key_values) in values.iter().enumerate() {
            let db_id = db.insert_node()?;
            ids.push(db_id);

            if let Some(alias) = self.aliases.get(index) {
                db.insert_new_alias(db_id, alias)?;
            }

            for key_value in *key_values {
                db.insert_key_value(db_id, &key_value.key, &key_value.value)?;
            }
        }

        result.result = ids.len() as i64;
        result.elements = ids
            .into_iter()
            .map(|id| DbElement {
                index: id,
                values: vec![],
            })
            .collect();

        Ok(())
    }
}
