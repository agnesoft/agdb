use super::graph_data_storage::GraphDataStorage;
use super::graph_data_storage_indexes::GraphDataStorageIndexes;
use super::graph_impl::GraphImpl;
use crate::storage::FileStorageData;
use crate::storage::Storage;
use crate::storage::StorageData;
use crate::storage::StorageVec;
use crate::DbError;

#[allow(dead_code)]
pub(crate) type StorageGraph<Data = FileStorageData> = GraphImpl<GraphDataStorage<Data>>;

impl<Data: StorageData> StorageGraph<Data> {
    pub(crate) fn storage_index(&self) -> i64 {
        self.data.index
    }
}

impl<Data: StorageData> TryFrom<std::rc::Rc<std::cell::RefCell<Storage<Data>>>>
    for StorageGraph<Data>
{
    type Error = DbError;

    fn try_from(
        storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    ) -> Result<Self, Self::Error> {
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

        Ok(StorageGraph {
            data: GraphDataStorage::<Data> {
                index,
                indexes,
                from,
                to,
                from_meta,
                to_meta,
            },
        })
    }
}

impl<Data: StorageData> TryFrom<(std::rc::Rc<std::cell::RefCell<Storage<Data>>>, i64)>
    for StorageGraph<Data>
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (std::rc::Rc<std::cell::RefCell<Storage<Data>>>, i64),
    ) -> Result<Self, Self::Error> {
        let indexes = storage_with_index
            .0
            .borrow_mut()
            .value::<GraphDataStorageIndexes>(storage_with_index.1)?;

        let from = StorageVec::<i64, Data>::try_from((storage_with_index.0.clone(), indexes.from))?;
        let to = StorageVec::<i64, Data>::try_from((storage_with_index.0.clone(), indexes.to))?;
        let from_meta =
            StorageVec::<i64, Data>::try_from((storage_with_index.0.clone(), indexes.from_meta))?;
        let to_meta =
            StorageVec::<i64, Data>::try_from((storage_with_index.0.clone(), indexes.to_meta))?;

        Ok(StorageGraph {
            data: GraphDataStorage::<Data> {
                index: storage_with_index.1,
                indexes,
                from,
                to,
                from_meta,
                to_meta,
            },
        })
    }
}
