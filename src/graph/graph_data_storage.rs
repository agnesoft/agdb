use super::graph_data::GraphData;
use super::graph_data_storage_indexes::GraphDataStorageIndexes;
use crate::storage::FileStorageData;
use crate::storage::Storage;
use crate::storage::StorageData;
use crate::storage::StorageVec;
use crate::DbError;

pub(crate) struct GraphDataStorage<Data = FileStorageData>
where
    Data: StorageData,
{
    pub(super) storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
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
    Data: StorageData,
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

    fn node_count(&self) -> Result<u64, crate::DbError> {
        Ok(self.to_meta.value(0)? as u64)
    }

    fn resize(&mut self, capacity: u64) -> Result<(), crate::DbError> {
        self.from.resize(capacity)?;
        self.to.resize(capacity)?;
        self.from_meta.resize(capacity)?;
        self.to_meta.resize(capacity)
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
