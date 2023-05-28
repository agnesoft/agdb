use super::graph_data::GraphData;
use super::graph_data_memory::GraphDataMemory;
use super::graph_data_storage_indexes::GraphDataStorageIndexes;
use super::graph_index::GraphIndex;
use crate::collections::vec::DbVec;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use std::cell::RefCell;
use std::rc::Rc;

pub struct GraphDataStorage<Data = FileStorage>
where
    Data: Storage,
{
    pub(crate) storage: Rc<RefCell<Data>>,
    pub(crate) storage_index: StorageIndex,
    #[allow(dead_code)]
    pub(crate) indexes: GraphDataStorageIndexes,
    pub(crate) from: DbVec<i64, Data>,
    pub(crate) to: DbVec<i64, Data>,
    pub(crate) from_meta: DbVec<i64, Data>,
    pub(crate) to_meta: DbVec<i64, Data>,
}

impl<Data> GraphDataStorage<Data>
where
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let id = storage.borrow_mut().transaction();

        let mut from = DbVec::<i64, Data>::new(storage.clone())?;
        from.push(&0)?;
        let mut to = DbVec::<i64, Data>::new(storage.clone())?;
        to.push(&0)?;
        let mut from_meta = DbVec::<i64, Data>::new(storage.clone())?;
        from_meta.push(&i64::MIN)?;
        let mut to_meta = DbVec::<i64, Data>::new(storage.clone())?;
        to_meta.push(&0)?;

        let indexes = GraphDataStorageIndexes {
            from: from.storage_index(),
            to: to.storage_index(),
            from_meta: from_meta.storage_index(),
            to_meta: to_meta.storage_index(),
        };

        let index = storage.borrow_mut().insert(&indexes)?;

        storage.borrow_mut().commit(id)?;

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

    pub fn from_storage(
        storage: Rc<RefCell<Data>>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let indexes = storage
            .borrow_mut()
            .value::<GraphDataStorageIndexes>(storage_index)?;

        let from = DbVec::<i64, Data>::from_storage(storage.clone(), indexes.from)?;
        let to = DbVec::<i64, Data>::from_storage(storage.clone(), indexes.to)?;
        let from_meta = DbVec::<i64, Data>::from_storage(storage.clone(), indexes.from_meta)?;
        let to_meta = DbVec::<i64, Data>::from_storage(storage.clone(), indexes.to_meta)?;

        Ok(GraphDataStorage::<Data> {
            storage,
            storage_index,
            indexes,
            from,
            to,
            from_meta,
            to_meta,
        })
    }

    pub fn to_graph_data_memory(&self) -> Result<GraphDataMemory, DbError> {
        Ok(GraphDataMemory {
            from: self.from.to_vec()?,
            to: self.to.to_vec()?,
            from_meta: self.from_meta.to_vec()?,
            to_meta: self.to_meta.to_vec()?,
        })
    }
}

impl<Data> GraphData for GraphDataStorage<Data>
where
    Data: Storage,
{
    fn capacity(&self) -> Result<u64, DbError> {
        Ok(self.from.len())
    }

    fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.storage.borrow_mut().commit(id)
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
        self.from.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn set_from_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.from_meta.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn set_node_count(&mut self, count: u64) -> Result<(), DbError> {
        self.to_meta.replace(0, &(count as i64))?;
        Ok(())
    }

    fn set_to(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.to.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn set_to_meta(&mut self, index: &GraphIndex, value: i64) -> Result<(), DbError> {
        self.to_meta.replace(index.as_u64(), &value)?;
        Ok(())
    }

    fn to(&self, index: &GraphIndex) -> Result<i64, DbError> {
        self.to.value(index.as_u64())
    }

    fn to_meta(&self, index: &GraphIndex) -> Result<i64, DbError> {
        self.to_meta.value(index.as_u64())
    }

    fn transaction(&mut self) -> u64 {
        self.storage.borrow_mut().transaction()
    }
}
