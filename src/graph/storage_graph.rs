use super::graph_data_storage::GraphDataStorage;
use super::graph_impl::GraphImpl;
use crate::storage::Storage;
use crate::storage::StorageData;
use crate::DbError;

#[allow(dead_code)]
pub(crate) type StorageGraph = GraphImpl<GraphDataStorage>;

impl<Data: StorageData> TryFrom<std::rc::Rc<std::cell::RefCell<Storage<Data>>>> for StorageGraph {
    type Error = DbError;

    fn try_from(
        storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<Data: StorageData> TryFrom<(std::rc::Rc<std::cell::RefCell<Storage<Data>>>, i64)>
    for StorageGraph
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (std::rc::Rc<std::cell::RefCell<Storage<Data>>>, i64),
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
