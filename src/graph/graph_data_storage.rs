use super::graph_data::GraphData;
use super::graph_data_storage_indexes::GraphDataStorageIndexes;
use crate::storage::StorageVec;
use agdb_db_error::DbError;
use agdb_storage::Storage;
use agdb_storage::StorageFile;

pub(crate) struct GraphDataStorage<Data = StorageFile>
where
    Data: Storage,
{
    pub(super) storage: std::rc::Rc<std::cell::RefCell<Data>>,
    pub(super) index: i64,
    #[allow(dead_code)]
    pub(super) indexes: GraphDataStorageIndexes,
    pub(super) from: StorageVec<i64, Data>,
    pub(super) to: StorageVec<i64, Data>,
    pub(super) from_meta: StorageVec<i64, Data>,
    pub(super) to_meta: StorageVec<i64, Data>,
}

impl<Data> GraphData for GraphDataStorage<Data>
where
    Data: Storage,
{
    fn capacity(&self) -> Result<u64, crate::DbError> {
        Ok(self.from.len())
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.storage.borrow_mut().commit()
    }

    fn free_index(&self) -> Result<i64, crate::DbError> {
        self.from_meta.value(0)
    }

    fn from(&self, index: i64) -> Result<i64, crate::DbError> {
        self.from.value(index as u64)
    }

    fn from_meta(&self, index: i64) -> Result<i64, crate::DbError> {
        self.from_meta.value(index as u64)
    }

    fn grow(&mut self) -> Result<(), crate::DbError> {
        self.from.push(&0)?;
        self.to.push(&0)?;
        self.from_meta.push(&0)?;
        self.to_meta.push(&0)
    }

    fn node_count(&self) -> Result<u64, crate::DbError> {
        Ok(self.to_meta.value(0)? as u64)
    }

    fn set_from(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        self.from.set_value(index as u64, &value)
    }

    fn set_from_meta(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        self.from_meta.set_value(index as u64, &value)
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), crate::DbError> {
        self.to_meta.set_value(0, &(count as i64))
    }

    fn set_to(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        self.to.set_value(index as u64, &value)
    }

    fn set_to_meta(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        self.to_meta.set_value(index as u64, &value)
    }

    fn to(&self, index: i64) -> Result<i64, crate::DbError> {
        self.to.value(index as u64)
    }

    fn to_meta(&self, index: i64) -> Result<i64, crate::DbError> {
        self.to_meta.value(index as u64)
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }
}
