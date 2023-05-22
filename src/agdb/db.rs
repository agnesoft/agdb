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
use crate::collections::dictionary_storage::DictionaryStorage;
use crate::collections::indexed_map::IndexedMap;
use crate::collections::indexed_map_storage::IndexedMapStorage;
use crate::collections::multi_map::MultiMap;
use crate::collections::multi_map_storage::MultiMapStorage;
use crate::command::Command;
use crate::graph::graph_index::GraphIndex;
use crate::graph::graph_storage::GraphStorage;
use crate::graph::Graph;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use crate::transaction_mut::TransactionMut;
use crate::utilities::serialize::Serialize;
use crate::DbId;
use crate::DbKey;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct DbStorageIndex {
    next_id: i64,
    graph: StorageIndex,
    aliases: (StorageIndex, StorageIndex),
    indexes: (StorageIndex, StorageIndex),
    dictionary: StorageIndex,
    values: StorageIndex,
}

impl Serialize for DbStorageIndex {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        todo!()
    }

    fn serialized_size(&self) -> u64 {
        todo!()
    }
}

struct DbStorage {
    storage: Rc<RefCell<FileStorage>>,
    storage_index: DbStorageIndex,
    graph: GraphStorage,
    aliases: IndexedMapStorage<String, DbId>,
    indexes: IndexedMapStorage<DbId, GraphIndex>,
    dictionary: DictionaryStorage<DbValue>,
    values: MultiMapStorage<DbId, DbKeyValueIndex>,
}

pub struct Db {
    storage: DbStorage,
    graph: Graph,
    aliases: IndexedMap<String, DbId>,
    indexes: IndexedMap<DbId, GraphIndex>,
    dictionary: Dictionary<DbValue>,
    values: MultiMap<DbId, DbKeyValueIndex>,
    next_id: i64,
    undo_stack: Vec<Command>,
}

impl Db {
    pub fn new(filename: &str) -> Result<Db, DbError> {
        let storage = Rc::new(RefCell::new(FileStorage::new(filename)?));
        let graph_storage;
        let aliases_storage;
        let indexes_storage;
        let dictionary_storage;
        let values_storage;

        if let Ok(index) = storage
            .borrow_mut()
            .value::<DbStorageIndex>(&StorageIndex::default())
        {
            todo!()
        } else if storage.borrow_mut().len()? == 0 {
            let index = storage.borrow_mut().insert(&DbStorageIndex::default())?;
            graph_storage = GraphStorage::new(storage.clone())?;
            aliases_storage = IndexedMapStorage::new(storage.clone())?;
            indexes_storage = IndexedMapStorage::new(storage.clone())?;
            dictionary_storage = DictionaryStorage::new(storage.clone())?;
            values_storage = MultiMapStorage::new(storage.clone())?;
            let storage_index = DbStorageIndex {
                next_id: 1,
                graph: graph_storage.storage_index(),
                aliases: aliases_storage.storage_index(),
                indexes: indexes_storage.storage_index(),
                dictionary: dictionary_storage.storage_index(),
                values: values_storage.storage_index(),
            };
            storage.borrow_mut().insert_at(&index, 0, &storage_index)?;
        } else {
            return Err(DbError::from(format!(
                "File '{filename}' is not a valid database file and is not empty."
            )));
        }

        Ok(Self {
            storage: DbStorage {
                storage,
                storage_index,
                graph: graph_storage,
                aliases: aliases_storage,
                indexes: indexes_storage,
                dictionary: dictionary_storage,
                values: values_storage,
            },
            graph: Graph::new(),
            aliases: IndexedMap::new(),
            indexes: IndexedMap::new(),
            dictionary: Dictionary::new(),
            values: MultiMap::new(),
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

    pub(crate) fn values(&self, db_id: DbId) -> Result<Vec<DbKeyValueIndex>, DbError> {
        self.values.values(&db_id)
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

    pub(crate) fn remove_keys(&mut self, db_id: DbId, keys: &[DbKey]) -> Result<i64, QueryError> {
        let mut result = 0;

        for key_value_index in self.values.values(&db_id)? {
            let key = self.value(&key_value_index.key)?;

            if keys.contains(&key) {
                let value = self.value(&key_value_index.value)?;
                self.remove_value(key_value_index.key)?;
                self.remove_value(key_value_index.value)?;
                self.values.remove_value(&db_id, &key_value_index)?;
                self.undo_stack.push(Command::InsertKeyValue {
                    id: db_id,
                    key_value: DbKeyValue { key, value },
                });
                result += 1;
            }
        }

        Ok(result)
    }

    fn remove_all_values(&mut self, db_id: DbId) -> Result<(), QueryError> {
        for key_value_index in self.values.values(&db_id)? {
            let key = self.value(&key_value_index.key)?;
            let value = self.value(&key_value_index.value)?;
            self.remove_value(key_value_index.key)?;
            self.remove_value(key_value_index.value)?;
            self.undo_stack.push(Command::InsertKeyValue {
                id: db_id,
                key_value: DbKeyValue { key, value },
            });
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
