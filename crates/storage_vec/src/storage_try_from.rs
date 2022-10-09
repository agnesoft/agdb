use crate::storage_vec::StorageVec;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

impl<T, Data> TryFrom<Rc<RefCell<Data>>> for StorageVec<T, Data>
where
    T: Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        let index = storage.borrow_mut().insert(&0_u64)?;

        Ok(Self {
            storage,
            storage_index: index,
            len: 0,
            capacity: 0,
            phantom_data: PhantomData,
        })
    }
}

impl<T: Serialize, Data: Storage> TryFrom<(Rc<RefCell<Data>>, StorageIndex)>
    for StorageVec<T, Data>
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        let byte_size = storage_with_index
            .0
            .borrow_mut()
            .value_size(&storage_with_index.1)?;
        let size = storage_with_index
            .0
            .borrow_mut()
            .value_at::<u64>(&storage_with_index.1, 0)?;

        Ok(Self {
            storage: storage_with_index.0,
            storage_index: storage_with_index.1,
            len: size,
            capacity: Self::capacity_from_bytes(byte_size),
            phantom_data: PhantomData,
        })
    }
}
