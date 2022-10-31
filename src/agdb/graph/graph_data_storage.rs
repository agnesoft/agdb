use super::graph_data::GraphData;
use super::graph_data_storage_indexes::GraphDataStorageIndexes;
use super::graph_index::GraphIndex;
use crate::collections::storage_vec::StorageVec;
use crate::db_error::DbError;
use crate::storage::storage_file::StorageFile;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
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

    fn from(&self, index: &GraphIndex) -> Result<i64, DbError> {
        self.from.value(index.as_u64())
    }

    fn from_meta(&self, index: &GraphIndex) -> Result<i64, DbError> {
        self.from_meta.value(index.as_u64())
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

    fn set_from(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.from.set_value(index.as_u64(), &value)
    }

    fn set_from_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.from_meta.set_value(index.as_u64(), &value)
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.to_meta.set_value(0, &(count as i64))
    }

    fn set_to(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.to.set_value(index.as_u64(), &value)
    }

    fn set_to_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.to_meta.set_value(index.as_u64(), &value)
    }

    fn to(&self, index: &GraphIndex) -> Result<i64, DbError> {
        self.to.value(index.as_u64())
    }

    fn to_meta(&self, index: &GraphIndex) -> Result<i64, DbError> {
        self.to_meta.value(index.as_u64())
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }
}

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
