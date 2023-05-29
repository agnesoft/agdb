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
use crate::collections::indexed_map::DbIndexedMap;
use crate::collections::multi_map::MultiMap;
use crate::collections::multi_map_storage::MultiMapStorage;
use crate::command::Command;
use crate::graph::DbGraph;
use crate::graph::GraphIndex;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::transaction_mut::TransactionMut;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
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
    graph: StorageIndex,
    aliases: (StorageIndex, StorageIndex),
    dictionary: StorageIndex,
    values: StorageIndex,
}

impl Serialize for DbStorageIndex {
    fn serialize(&self) -> Vec<u8> {
        [
            self.graph.serialize(),
            self.aliases.0.serialize(),
            self.aliases.1.serialize(),
            self.dictionary.serialize(),
            self.values.serialize(),
        ]
        .concat()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        let size = i64::serialized_size_static() as usize;

        let graph = StorageIndex::deserialize(bytes)?;
        let aliases_1 = StorageIndex::deserialize(&bytes[size..])?;
        let aliases_2 = StorageIndex::deserialize(&bytes[size * 2..])?;
        let dictionary = StorageIndex::deserialize(&bytes[size * 3..])?;
        let values = StorageIndex::deserialize(&bytes[size * 4..])?;

        Ok(Self {
            graph,
            aliases: (aliases_1, aliases_2),
            dictionary,
            values,
        })
    }

    fn serialized_size(&self) -> u64 {
        i64::serialized_size_static() * 5
    }
}

struct DbStorage {
    _storage: Rc<RefCell<FileStorage>>,
    graph: DbGraph,
    aliases: DbIndexedMap<String, DbId>,
    dictionary: DictionaryStorage<DbValue>,
    values: MultiMapStorage<DbId, DbKeyValueIndex>,
}

pub struct Db {
    storage: DbStorage,
    dictionary: Dictionary<DbValue>,
    values: MultiMap<DbId, DbKeyValueIndex>,
    undo_stack: Vec<Command>,
}

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db").finish_non_exhaustive()
    }
}

impl Db {
    pub fn new(filename: &str) -> Result<Db, DbError> {
        match Self::try_new(filename) {
            Ok(db) => Ok(db),
            Err(error) => {
                let mut db_error = DbError::from("Failed to create database");
                db_error.cause = Some(Box::new(error));
                Err(db_error)
            }
        }
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
                Command::InsertAlias { id, alias } => {
                    self.storage.aliases.insert(alias, id)?;
                }
                Command::InsertEdge { from, to } => {
                    self.storage.graph.insert_edge(*from, *to)?;
                }
                Command::InsertKeyValue { id, key_value } => {
                    let key = self.insert_value(&key_value.key)?;
                    let value = self.insert_value(&key_value.value)?;
                    self.values.insert(id, &DbKeyValueIndex { key, value })?;
                    self.storage
                        .values
                        .insert(id, &DbKeyValueIndex { key, value })?;
                }
                Command::InsertNode => {
                    self.storage.graph.insert_node()?;
                }
                Command::RemoveAlias { alias } => {
                    self.storage.aliases.remove_key(alias)?;
                }
                Command::RemoveEdge { index } => {
                    self.storage.graph.remove_edge(*index)?;
                }
                Command::RemoveKeyValue { id, key_value } => {
                    self.values.remove_value(id, key_value)?;
                    self.storage.values.remove_value(id, key_value)?;
                }
                Command::RemoveNode { index } => {
                    self.storage.graph.remove_node(*index)?;
                }
                Command::RemoveValue { index } => {
                    self.dictionary.remove(*index)?;
                    self.storage.dictionary.remove(*index)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, QueryError> {
        match query_id {
            QueryId::Id(id) => Ok(DbId(self.graph_index(id.0)?.0)),
            QueryId::Alias(alias) => Ok(self
                .storage
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?),
        }
    }

    pub(crate) fn insert_alias(&mut self, db_id: DbId, alias: &String) -> Result<(), DbError> {
        if let Some(old_alias) = self.storage.aliases.key(&db_id)? {
            self.undo_stack.push(Command::InsertAlias {
                id: db_id,
                alias: old_alias.clone(),
            });
            self.storage.aliases.remove_key(&old_alias)?;
            self.storage.aliases.remove_key(&old_alias)?;
        }

        self.undo_stack.push(Command::RemoveAlias {
            alias: alias.clone(),
        });
        self.storage.aliases.insert(alias, &db_id)
    }

    pub(crate) fn insert_edge(&mut self, from: &QueryId, to: &QueryId) -> Result<DbId, QueryError> {
        let from = self.db_id(from)?;
        let to = self.db_id(to)?;

        let index = self
            .storage
            .graph
            .insert_edge(GraphIndex(from.0), GraphIndex(to.0))?;
        self.undo_stack.push(Command::RemoveEdge { index });

        Ok(DbId(index.0))
    }

    pub(crate) fn insert_new_alias(
        &mut self,
        db_id: DbId,
        alias: &String,
    ) -> Result<(), QueryError> {
        if let Some(id) = self.storage.aliases.value(alias)? {
            return Err(QueryError::from(format!(
                "Alias '{alias}' already exists ({})",
                id.0
            )));
        }

        self.undo_stack.push(Command::RemoveAlias {
            alias: alias.clone(),
        });
        self.storage.aliases.insert(alias, &db_id)?;

        Ok(())
    }

    pub(crate) fn insert_node(&mut self) -> Result<DbId, DbError> {
        let index = self.storage.graph.insert_node()?;
        self.undo_stack.push(Command::RemoveNode { index });

        Ok(DbId(index.0))
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

        self.undo_stack.push(Command::RemoveKeyValue {
            id: db_id,
            key_value,
        });
        self.values.insert(&db_id, &key_value)?;
        self.storage.values.insert(&db_id, &key_value)?;

        Ok(())
    }

    pub(crate) fn remove(&mut self, query_id: &QueryId) -> Result<bool, QueryError> {
        match query_id {
            QueryId::Id(db_id) => {
                if let Ok(graph_index) = self.graph_index(db_id.0) {
                    if graph_index.is_node() {
                        self.remove_node(*db_id, graph_index, self.storage.aliases.key(db_id)?)?;
                    } else {
                        self.remove_edge(graph_index)?;
                    }
                    self.remove_all_values(*db_id)?;
                    return Ok(true);
                }
            }
            QueryId::Alias(alias) => {
                if let Some(db_id) = self.storage.aliases.value(alias)? {
                    self.remove_node(db_id, GraphIndex(db_id.0), Some(alias.clone()))?;
                    self.remove_all_values(db_id)?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub(crate) fn remove_alias(&mut self, alias: &String) -> Result<bool, DbError> {
        if let Some(id) = self.storage.aliases.value(alias)? {
            self.undo_stack.push(Command::InsertAlias {
                id,
                alias: alias.clone(),
            });
            self.storage.aliases.remove_key(alias)?;
            self.storage.aliases.remove_key(alias)?;

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

    fn graph_index(&self, id: i64) -> Result<GraphIndex, QueryError> {
        match id.cmp(&0) {
            std::cmp::Ordering::Less => {
                if self.storage.graph.edge(GraphIndex(id)).is_some() {
                    return Ok(GraphIndex(id));
                }
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                if self.storage.graph.node(GraphIndex(id)).is_some() {
                    return Ok(GraphIndex(id));
                }
            }
        }

        Err(QueryError::from(format!("Id '{id}' not found")))
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
        let _ = self.storage.dictionary.insert(value)?;
        index.set_index(dictionary_index.0);
        Ok(index)
    }

    fn node_edges(
        &self,
        graph_index: GraphIndex,
    ) -> Result<Vec<(GraphIndex, GraphIndex, GraphIndex)>, DbError> {
        let mut edges = vec![];
        let node = self
            .storage
            .graph
            .node(graph_index)
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

    fn remove_edge(&mut self, graph_index: GraphIndex) -> Result<(), DbError> {
        let (from, to) = {
            let edge = self
                .storage
                .graph
                .edge(graph_index)
                .ok_or(DbError::from("Graph integrity corrupted"))?;
            (edge.index_from(), edge.index_to())
        };

        self.storage.graph.remove_edge(graph_index)?;
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
            self.undo_stack.push(Command::InsertAlias {
                alias: alias.clone(),
                id: db_id,
            });
            self.storage.aliases.remove_key(&alias)?;
            self.storage.aliases.remove_key(&alias)?;
        }

        for edge in self.node_edges(graph_index)? {
            self.storage.graph.remove_edge(edge.0)?;
            self.undo_stack.push(Command::InsertEdge {
                from: edge.1,
                to: edge.2,
            });
        }

        self.storage.graph.remove_node(graph_index)?;
        self.undo_stack.push(Command::InsertNode);
        Ok(())
    }

    fn remove_value(&mut self, value_index: DbValueIndex) -> Result<(), DbError> {
        if !value_index.is_value() {
            self.dictionary
                .remove(DictionaryIndex(value_index.index()))?;
            self.storage
                .dictionary
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

                self.undo_stack.push(Command::InsertKeyValue {
                    id: db_id,
                    key_value: DbKeyValue { key, value },
                });
                self.values.remove_value(&db_id, &key_value_index)?;
                self.storage.values.remove_value(&db_id, &key_value_index)?;

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

    fn try_new(filename: &str) -> Result<Db, DbError> {
        let storage = Rc::new(RefCell::new(FileStorage::new(filename)?));
        let graph_storage;
        let aliases_storage;
        let dictionary;
        let dictionary_storage;
        let values;
        let values_storage;
        let len = storage.borrow_mut().len()?;
        let index = storage
            .borrow_mut()
            .value::<DbStorageIndex>(StorageIndex(1));

        if let Ok(index) = index {
            graph_storage = DbGraph::from_storage(storage.clone(), index.graph)?;
            aliases_storage = DbIndexedMap::from_storage(storage.clone(), index.aliases)?;
            dictionary_storage =
                DictionaryStorage::from_storage(storage.clone(), index.dictionary)?;
            dictionary = dictionary_storage.to_dictionary()?;
            values_storage = MultiMapStorage::from_storage(storage.clone(), index.values)?;
            values = values_storage.to_multi_map()?;
        } else if len == 0 {
            storage.borrow_mut().insert(&DbStorageIndex::default())?;
            graph_storage = DbGraph::new(storage.clone())?;
            aliases_storage = DbIndexedMap::new(storage.clone())?;
            dictionary = Dictionary::new();
            dictionary_storage = DictionaryStorage::new(storage.clone())?;
            values = MultiMap::new();
            values_storage = MultiMapStorage::new(storage.clone())?;
            let db_storage_index = DbStorageIndex {
                graph: graph_storage.storage_index(),
                aliases: aliases_storage.storage_index(),
                dictionary: dictionary_storage.storage_index(),
                values: values_storage.storage_index(),
            };
            storage
                .borrow_mut()
                .insert_at(StorageIndex(1), 0, &db_storage_index)?;
        } else {
            return Err(DbError::from(format!(
                "File '{filename}' is not a valid database file and is not empty."
            )));
        }

        Ok(Self {
            storage: DbStorage {
                _storage: storage,
                graph: graph_storage,
                aliases: aliases_storage,
                dictionary: dictionary_storage,
                values: values_storage,
            },
            dictionary,
            values,
            undo_stack: vec![],
        })
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

    #[test]
    fn db_storage_index_serialized_size() {
        assert_eq!(DbStorageIndex::default().serialized_size(), 40);
    }

    #[test]
    fn derived_from_debug() {
        let test_file = TestFile::new();
        let db = Db::new(test_file.file_name()).unwrap();
        format!("{:?}", db);
    }
}
