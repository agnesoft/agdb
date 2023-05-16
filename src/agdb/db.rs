pub mod db_element;
pub mod db_error;
pub mod db_id;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

pub(crate) mod db_key_value_index;
pub(crate) mod db_value_index;

mod db_float;

use self::db_error::DbError;
use self::db_float::DbFloat;
use self::db_key_value_index::DbKeyValueIndex;
use self::db_value::BYTES_META_VALUE;
use self::db_value::FLOAT_META_VALUE;
use self::db_value::INT_META_VALUE;
use self::db_value::STRING_META_VALUE;
use self::db_value::UINT_META_VALUE;
use self::db_value_index::DbValueIndex;
use crate::collections::dictionary::dictionary_index::DictionaryIndex;
use crate::collections::dictionary::Dictionary;
use crate::collections::indexed_map::IndexedMap;
use crate::collections::multi_map::MultiMap;
use crate::command::Command;
use crate::graph::graph_index::GraphIndex;
use crate::graph::Graph;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::transaction_mut::TransactionMut;
use crate::DbId;
use crate::DbKey;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

pub struct Db {
    pub(crate) graph: Graph,
    pub(crate) aliases: IndexedMap<String, DbId>,
    pub(crate) indexes: IndexedMap<DbId, GraphIndex>,
    pub(crate) dictionary: Dictionary<DbValue>,
    pub(crate) values: MultiMap<DbId, DbKeyValueIndex>,
    pub(crate) next_id: i64,
    undo_stack: Vec<Command>,
}

impl Db {
    pub fn new(_filename: &str) -> Result<Db, DbError> {
        Ok(Self {
            graph: Graph::new(),
            aliases: IndexedMap::<String, DbId>::new(),
            indexes: IndexedMap::<DbId, GraphIndex>::new(),
            dictionary: Dictionary::<DbValue>::new(),
            values: MultiMap::<DbId, DbKeyValueIndex>::new(),
            next_id: 1,
            undo_stack: vec![],
        })
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction(|transaction| transaction.exec(query))
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    pub fn transaction<T, E>(&self, f: impl Fn(&Transaction) -> Result<T, E>) -> Result<T, E> {
        let transaction = Transaction::new(self);

        f(&transaction)
    }

    pub fn transaction_mut<T, E: From<QueryError>>(
        &mut self,
        f: impl Fn(&mut TransactionMut) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut transaction = TransactionMut::new(&mut *self);
        let result = f(&mut transaction);

        if result.is_ok() {
            transaction.commit()?;
        } else {
            transaction.rollback()?;
        }

        result
    }

    pub(crate) fn commit(&mut self) -> Result<(), QueryError> {
        self.undo_stack.clear();
        Ok(())
    }

    pub(crate) fn rollback(&mut self) -> Result<(), QueryError> {
        let mut undo_stack = vec![];
        std::mem::swap(&mut undo_stack, &mut self.undo_stack);

        for command in undo_stack.iter().rev() {
            match command {
                Command::InsertAlias { id, alias } => self.aliases.insert(alias, id)?,
                Command::InsertEdge { from, to } => {
                    self.graph.insert_edge(from, to)?;
                }
                Command::InsertId { id, graph_index } => self.indexes.insert(id, graph_index)?,
                Command::InsertKeyValue { id, key_value } => {
                    let key = self.insert_value(&key_value.key)?;
                    let value = self.insert_value(&key_value.value)?;
                    self.values.insert(id, &DbKeyValueIndex { key, value })?;
                }
                Command::InsertNode => {
                    self.graph.insert_node()?;
                }
                Command::NextId { id } => self.next_id = *id,
                Command::RemoveAlias { alias } => self.aliases.remove_key(alias)?,
                Command::RemoveId { id } => self.indexes.remove_key(id)?,
                Command::RemoveEdge { index } => self.graph.remove_edge(index)?,
                Command::RemoveKeyValue { id, key_value } => {
                    self.values.remove_value(id, key_value)?
                }
                Command::RemoveNode { index } => self.graph.remove_node(index)?,
                Command::RemoveValue { index } => {
                    self.dictionary.remove(*index)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, QueryError> {
        Ok(match query_id {
            QueryId::Id(id) => {
                let _ = self
                    .indexes
                    .value(id)?
                    .ok_or(QueryError::from(format!("Id '{}' not found", id.0)))?;
                *id
            }
            QueryId::Alias(alias) => self
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        })
    }

    pub(crate) fn graph_index_from_id(&self, id: &QueryId) -> Result<GraphIndex, QueryError> {
        let db_id = match id {
            QueryId::Id(id) => *id,
            QueryId::Alias(alias) => self
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        };

        self.indexes
            .value(&db_id)?
            .ok_or(QueryError::from(format!("Id '{}' not found", db_id.0)))
    }

    pub(crate) fn insert_alias(&mut self, db_id: DbId, alias: &String) -> Result<(), DbError> {
        if let Some(old_alias) = self.aliases.key(&db_id)? {
            self.undo_stack.push(Command::InsertAlias {
                id: db_id,
                alias: old_alias.clone(),
            });

            self.aliases.remove_key(&old_alias)?;
        }

        self.undo_stack.push(Command::RemoveAlias {
            alias: alias.clone(),
        });
        self.aliases.insert(alias, &db_id)?;

        Ok(())
    }

    pub(crate) fn insert_edge(&mut self, from: &QueryId, to: &QueryId) -> Result<DbId, QueryError> {
        let from = self.graph_index_from_id(from)?;
        let to = self.graph_index_from_id(to)?;
        let graph_index = self.graph.insert_edge(&from, &to)?;
        self.undo_stack
            .push(Command::RemoveEdge { index: graph_index });
        let db_id = DbId(-self.next_id);
        self.undo_stack.push(Command::NextId { id: self.next_id });
        self.next_id += 1;
        self.undo_stack.push(Command::RemoveId { id: db_id });
        self.indexes.insert(&db_id, &graph_index)?;
        Ok(db_id)
    }

    pub(crate) fn insert_new_alias(
        &mut self,
        db_id: DbId,
        alias: &String,
    ) -> Result<(), QueryError> {
        if let Some(id) = self.aliases.value(alias)? {
            return Err(QueryError::from(format!(
                "Alias '{alias}' already exists ({})",
                id.0
            )));
        }

        self.undo_stack.push(Command::RemoveAlias {
            alias: alias.clone(),
        });
        self.aliases.insert(alias, &db_id)?;

        Ok(())
    }

    pub(crate) fn insert_node(&mut self) -> Result<DbId, DbError> {
        let graph_index = self.graph.insert_node()?;
        self.undo_stack
            .push(Command::RemoveNode { index: graph_index });
        let db_id = DbId(self.next_id);
        self.undo_stack.push(Command::NextId { id: self.next_id });
        self.next_id += 1;
        self.undo_stack.push(Command::RemoveId { id: db_id });
        self.indexes.insert(&db_id, &graph_index)?;
        Ok(db_id)
    }

    pub(crate) fn insert_key_value(
        &mut self,
        db_id: DbId,
        key: &DbKey,
        value: &DbValue,
    ) -> Result<(), QueryError> {
        let key = self.insert_value(key)?;

        if !key.is_value() {
            self.undo_stack.push(Command::RemoveValue {
                index: DictionaryIndex(key.index()),
            });
        }

        let value = self.insert_value(value)?;

        if !key.is_value() {
            self.undo_stack.push(Command::RemoveValue {
                index: DictionaryIndex(value.index()),
            });
        }

        let key_value = DbKeyValueIndex { key, value };
        self.values.insert(&db_id, &key_value)?;
        self.undo_stack.push(Command::RemoveKeyValue {
            id: db_id,
            key_value,
        });

        Ok(())
    }

    pub(crate) fn remove(&mut self, query_id: &QueryId) -> Result<bool, QueryError> {
        match query_id {
            QueryId::Id(db_id) => {
                if let Some(graph_index) = self.indexes.value(db_id)? {
                    if graph_index.is_node() {
                        self.remove_node(*db_id, graph_index, self.aliases.key(db_id)?)?;
                    } else {
                        self.remove_edge(*db_id, graph_index)?;
                    }
                    self.remove_all_values(*db_id)?;
                    return Ok(true);
                }
            }
            QueryId::Alias(alias) => {
                if let Some(db_id) = self.aliases.value(alias)? {
                    let graph_index = self
                        .indexes
                        .value(&db_id)?
                        .ok_or(DbError::from("Data integrity corrupted"))?;
                    self.remove_node(db_id, graph_index, Some(alias.clone()))?;
                    self.remove_all_values(db_id)?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub(crate) fn remove_alias(&mut self, alias: &String) -> Result<bool, DbError> {
        if let Some(id) = self.aliases.value(alias)? {
            self.aliases.remove_key(alias)?;
            self.undo_stack.push(Command::InsertAlias {
                id,
                alias: alias.clone(),
            });

            return Ok(true);
        }

        Ok(false)
    }

    pub(crate) fn value(&self, index: &DbValueIndex) -> Result<DbValue, QueryError> {
        if index.is_value() {
            match index.get_type() {
                BYTES_META_VALUE => {
                    return Ok(DbValue::Bytes(index.value().to_vec()));
                }
                INT_META_VALUE => {
                    let mut bytes = [0_u8; 8];
                    bytes.copy_from_slice(index.value());
                    return Ok(DbValue::Int(i64::from_le_bytes(bytes)));
                }
                UINT_META_VALUE => {
                    let mut bytes = [0_u8; 8];
                    bytes.copy_from_slice(index.value());
                    return Ok(DbValue::Uint(u64::from_le_bytes(bytes)));
                }
                FLOAT_META_VALUE => {
                    let mut bytes = [0_u8; 8];
                    bytes.copy_from_slice(index.value());
                    return Ok(DbValue::Float(DbFloat::from(f64::from_le_bytes(bytes))));
                }
                STRING_META_VALUE => {
                    return Ok(DbValue::String(
                        String::from_utf8_lossy(index.value()).to_string(),
                    ))
                }
                _ => panic!(),
            }
        }

        let dictionary_index = DictionaryIndex(index.index());
        let value = self
            .dictionary
            .value(dictionary_index)?
            .ok_or(QueryError::from(format!(
                "Value not found (index: '{}')",
                dictionary_index.0
            )))?;
        Ok(value)
    }

    fn insert_value(&mut self, value: &DbValue) -> Result<DbValueIndex, QueryError> {
        let mut index = DbValueIndex::new();

        match value {
            DbValue::Bytes(v) => {
                index.set_type(db_value::BYTES_META_VALUE);
                if index.set_value(v) {
                    return Ok(index);
                }
            }
            DbValue::Int(v) => {
                index.set_type(db_value::INT_META_VALUE);
                index.set_value(&v.to_le_bytes());
                return Ok(index);
            }
            DbValue::Uint(v) => {
                index.set_type(db_value::UINT_META_VALUE);
                index.set_value(&v.to_le_bytes());
                return Ok(index);
            }
            DbValue::Float(v) => {
                index.set_type(db_value::FLOAT_META_VALUE);
                index.set_value(&v.to_f64().to_le_bytes());
                return Ok(index);
            }
            DbValue::String(v) => {
                index.set_type(db_value::STRING_META_VALUE);
                let bytes = v.as_bytes();
                if index.set_value(bytes) {
                    return Ok(index);
                }
            }
            DbValue::VecInt(_) => {
                index.set_type(db_value::VEC_INT_META_VALUE);
            }
            DbValue::VecUint(_) => {
                index.set_type(db_value::VEC_UINT_META_VALUE);
            }
            DbValue::VecFloat(_) => {
                index.set_type(db_value::VEC_FLOAT_META_VALUE);
            }
            DbValue::VecString(_) => {
                index.set_type(db_value::VEC_STRING_META_VALUE);
            }
        }

        let dictionary_index = self.dictionary.insert(value)?;
        index.set_index(dictionary_index.0);
        Ok(index)
    }

    fn node_edges(
        &self,
        graph_index: GraphIndex,
    ) -> Result<Vec<(GraphIndex, GraphIndex, GraphIndex)>, DbError> {
        let mut edges = vec![];
        let node = self
            .graph
            .node(&graph_index)
            .ok_or(DbError::from("Data integrity corrupted"))?;

        for edge in node.edge_iter_from() {
            edges.push((edge.index, edge.index_from(), edge.index_to()));
        }

        for edge in node.edge_iter_to() {
            let from = edge.index_from();
            if from != graph_index {
                edges.push((edge.index, from, edge.index_to()));
            }
        }

        Ok(edges)
    }

    fn remove_edge(&mut self, db_id: DbId, graph_index: GraphIndex) -> Result<(), DbError> {
        let (from, to) = {
            let edge = self
                .graph
                .edge(&graph_index)
                .ok_or(DbError::from("Data integrity corrupted"))?;
            (edge.index_from(), edge.index_to())
        };
        self.indexes.remove_key(&db_id)?;
        self.undo_stack.push(Command::InsertId {
            id: db_id,
            graph_index,
        });
        self.graph.remove_edge(&graph_index)?;
        self.undo_stack.push(Command::InsertEdge { from, to });

        Ok(())
    }

    fn remove_node(
        &mut self,
        db_id: DbId,
        graph_index: GraphIndex,
        alias: Option<String>,
    ) -> Result<(), DbError> {
        if let Some(alias) = alias {
            self.aliases.remove_key(&alias)?;
            self.undo_stack.push(Command::InsertAlias {
                id: db_id,
                alias: alias.clone(),
            });
        }

        self.indexes.remove_key(&db_id)?;
        self.undo_stack.push(Command::InsertId {
            id: db_id,
            graph_index,
        });

        for edge in self.node_edges(graph_index)? {
            self.remove_node_edge(edge.0, edge.1, edge.2)?;
        }

        self.graph.remove_node(&graph_index)?;
        self.undo_stack.push(Command::InsertNode);

        Ok(())
    }

    fn remove_node_edge(
        &mut self,
        graph_index: GraphIndex,
        from: GraphIndex,
        to: GraphIndex,
    ) -> Result<(), DbError> {
        let db_id = self
            .indexes
            .key(&graph_index)?
            .ok_or(DbError::from("Data integrity corrupted"))?;
        self.indexes.remove_key(&db_id)?;
        self.undo_stack.push(Command::InsertId {
            id: db_id,
            graph_index,
        });
        self.graph.remove_edge(&graph_index)?;
        self.undo_stack.push(Command::InsertEdge { from, to });
        Ok(())
    }

    fn remove_value(&mut self, value_index: DbValueIndex) -> Result<(), DbError> {
        if !value_index.is_value() {
            self.dictionary
                .remove(DictionaryIndex(value_index.index()))?;
        }

        Ok(())
    }

    fn remove_all_values(&mut self, db_id: DbId) -> Result<(), QueryError> {
        for key_value_index in self.values.values(&db_id)? {
            let key = self.value(&key_value_index.key)?;
            let value = self.value(&key_value_index.value)?;
            self.undo_stack.push(Command::InsertKeyValue {
                id: db_id,
                key_value: DbKeyValue { key, value },
            });
            self.remove_value(key_value_index.key)?;
            self.remove_value(key_value_index.value)?;
        }

        self.values.remove_key(&db_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    #[should_panic]
    fn invalid_value_type() {
        let test_file = TestFile::new();
        let db = Db::new(test_file.file_name()).unwrap();

        let mut index = DbValueIndex::new();
        index.set_type(15_u8);
        index.set_value(&1_u64.to_le_bytes());

        db.value(&index).unwrap();
    }
}
