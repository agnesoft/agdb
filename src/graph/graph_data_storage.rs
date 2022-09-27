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
    fn capacity(&self) -> u64 {
        todo!()
    }

    fn free_index(&self) -> i64 {
        todo!()
    }

    fn from(&self, index: i64) -> i64 {
        todo!()
    }

    fn from_meta(&self, index: i64) -> i64 {
        todo!()
    }

    fn node_count(&self) -> u64 {
        todo!()
    }

    fn resize(&mut self, capacity: u64) {
        todo!()
    }

    fn set_from(&mut self, index: i64, value: i64) {
        todo!()
    }

    fn set_from_meta(&mut self, index: i64, value: i64) {
        todo!()
    }

    fn set_node_count(&mut self, count: u64) {
        todo!()
    }

    fn set_to(&mut self, index: i64, value: i64) {
        todo!()
    }

    fn set_to_meta(&mut self, index: i64, value: i64) {
        todo!()
    }

    fn to(&self, index: i64) -> i64 {
        todo!()
    }

    fn to_meta(&self, index: i64) -> i64 {
        todo!()
    }
}
