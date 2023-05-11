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
use crate::graph::graph_index::GraphIndex;
use crate::graph::Graph;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::transaction_mut::TransactionMut;
use crate::DbId;
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
        Self::finish_transaction(transaction, result.is_ok())?;
        result
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

    pub(crate) fn insert_value(&mut self, value: &DbValue) -> Result<DbValueIndex, QueryError> {
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

        index.set_index(self.dictionary.insert(value)?.0);
        Ok(index)
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

    fn finish_transaction(transaction: TransactionMut, result: bool) -> Result<(), QueryError> {
        if result {
            transaction.commit()
        } else {
            transaction.rollback()
        }
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
        let db = Db::new(&test_file.file_name()).unwrap();

        let mut index = DbValueIndex::new();
        index.set_type(15_u8);
        index.set_value(&1_u64.to_le_bytes());

        db.value(&index).unwrap();
    }
}
