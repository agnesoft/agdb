use crate::storage::FileStorageData;
use crate::storage::Storage;
use crate::storage::StorageData;

use super::graph_data::GraphData;

pub(crate) struct GraphDataStorage<Data = FileStorageData>
where
    Data: StorageData,
{
    storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
}

impl<Data> GraphData for GraphDataStorage<Data>
where
    Data: StorageData,
{
    fn capacity(&self) -> Result<u64, crate::DbError> {
        todo!()
    }

    fn free_index(&self) -> Result<i64, crate::DbError> {
        todo!()
    }

    fn from(&self, index: i64) -> Result<i64, crate::DbError> {
        todo!()
    }

    fn from_meta(&self, index: i64) -> Result<i64, crate::DbError> {
        todo!()
    }

    fn node_count(&self) -> Result<u64, crate::DbError> {
        todo!()
    }

    fn resize(&mut self, capacity: u64) -> Result<(), crate::DbError> {
        todo!()
    }

    fn set_from(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        todo!()
    }

    fn set_from_meta(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        todo!()
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), crate::DbError> {
        todo!()
    }

    fn set_to(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        todo!()
    }

    fn set_to_meta(&mut self, index: i64, value: i64) -> Result<(), crate::DbError> {
        todo!()
    }

    fn to(&self, index: i64) -> Result<i64, crate::DbError> {
        todo!()
    }

    fn to_meta(&self, index: i64) -> Result<i64, crate::DbError> {
        todo!()
    }
}
