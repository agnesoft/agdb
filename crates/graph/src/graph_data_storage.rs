use super::graph_data::GraphData;
use super::graph_data_storage_indexes::GraphDataStorageIndexes;
use agdb_db_error::DbError;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_storage_vec::StorageVec;
use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphDataStorage<Data = StorageFile>
where
    Data: Storage,
{
    pub(crate) storage: Rc<RefCell<Data>>,
    pub(crate) storage_index: StorageIndex,
    #[allow(dead_code)]
    pub(crate) indexes: GraphDataStorageIndexes,
    pub(crate) from: StorageVec<i64, Data>,
    pub(crate) to: StorageVec<i64, Data>,
    pub(crate) from_meta: StorageVec<i64, Data>,
    pub(crate) to_meta: StorageVec<i64, Data>,
}

impl<Data> GraphData for GraphDataStorage<Data>
where
    Data: Storage,
{
    fn capacity(&self) -> Result<u64, DbError> {
        Ok(self.from.len())
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.storage.borrow_mut().commit()
    }

    fn free_index(&self) -> Result<i64, DbError> {
        self.from_meta.value(0)
    }

    fn from(&self, index: i64) -> Result<i64, DbError> {
        self.from.value(index as u64)
    }

    fn from_meta(&self, index: i64) -> Result<i64, DbError> {
        self.from_meta.value(index as u64)
    }

    fn grow(&mut self) -> Result<(), DbError> {
        self.from.push(&0)?;
        self.to.push(&0)?;
        self.from_meta.push(&0)?;
        self.to_meta.push(&0)
    }

    fn node_count(&self) -> Result<u64, DbError> {
        Ok(self.to_meta.value(0)? as u64)
    }

    fn set_from(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.from.set_value(index as u64, &value)
    }

    fn set_from_meta(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.from_meta.set_value(index as u64, &value)
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.to_meta.set_value(0, &(count as i64))
    }

    fn set_to(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.to.set_value(index as u64, &value)
    }

    fn set_to_meta(&mut self, index: i64, value: i64) -> Result<(), DbError> {
        self.to_meta.set_value(index as u64, &value)
    }

    fn to(&self, index: i64) -> Result<i64, DbError> {
        self.to.value(index as u64)
    }

    fn to_meta(&self, index: i64) -> Result<i64, DbError> {
        self.to_meta.value(index as u64)
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }
}
