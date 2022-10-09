use crate::graph_data_storage::GraphDataStorage;
use crate::graph_data_storage_indexes::GraphDataStorageIndexes;
use agdb_db_error::DbError;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use agdb_storage_vec::StorageVec;
use std::cell::RefCell;
use std::rc::Rc;

impl<Data> TryFrom<Rc<RefCell<Data>>> for GraphDataStorage<Data>
where
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        let mut from = StorageVec::<i64, Data>::try_from(storage.clone())?;
        from.push(&0)?;
        let mut to = StorageVec::<i64, Data>::try_from(storage.clone())?;
        to.push(&0)?;
        let mut from_meta = StorageVec::<i64, Data>::try_from(storage.clone())?;
        from_meta.push(&i64::MIN)?;
        let mut to_meta = StorageVec::<i64, Data>::try_from(storage.clone())?;
        to_meta.push(&0)?;

        let indexes = GraphDataStorageIndexes {
            from: from.storage_index(),
            to: to.storage_index(),
            from_meta: from_meta.storage_index(),
            to_meta: to_meta.storage_index(),
        };

        let index = storage.borrow_mut().insert(&indexes)?;

        Ok(GraphDataStorage::<Data> {
            storage,
            storage_index: index,
            indexes,
            from,
            to,
            from_meta,
            to_meta,
        })
    }
}

impl<Data: Storage> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for GraphDataStorage<Data> {
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        let indexes = storage_with_index
            .0
            .borrow_mut()
            .value::<GraphDataStorageIndexes>(&storage_with_index.1)?;

        let from = StorageVec::<i64, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.from.clone(),
        ))?;
        let to =
            StorageVec::<i64, Data>::try_from((storage_with_index.0.clone(), indexes.to.clone()))?;
        let from_meta = StorageVec::<i64, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.from_meta.clone(),
        ))?;
        let to_meta = StorageVec::<i64, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.to_meta.clone(),
        ))?;

        Ok(GraphDataStorage::<Data> {
            storage: storage_with_index.0,
            storage_index: storage_with_index.1,
            indexes,
            from,
            to,
            from_meta,
            to_meta,
        })
    }
}
