use super::graph_data_storage::GraphDataStorage;
use crate::StorageGraph;
use agdb_db_error::DbError;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use std::cell::RefCell;
use std::rc::Rc;

impl<Data> TryFrom<Rc<RefCell<Data>>> for StorageGraph<Data>
where
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        Ok(StorageGraph {
            data: GraphDataStorage::<Data>::try_from(storage)?,
        })
    }
}

impl<Data: Storage> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageGraph<Data> {
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        Ok(StorageGraph {
            data: GraphDataStorage::<Data>::try_from(storage_with_index)?,
        })
    }
}
