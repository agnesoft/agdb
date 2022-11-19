pub mod vec_storage_dynamic_sized;
pub mod vec_storage_fixed_sized;

use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeFixedSized;
use crate::DbError;
use crate::DbIndex;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct VecStorage<T, Data = FileStorage>
where
    T: Serialize,
    Data: Storage,
{
    phantom_data: PhantomData<T>,
    storage: Rc<RefCell<Data>>,
    storage_index: DbIndex,
    indexes: Vec<DbIndex>,
    len: u64,
    capacity: u64,
}

#[allow(dead_code)]
impl<T, Data> VecStorage<T, Data>
where
    T: Serialize,
    Data: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let storage_index = storage.borrow_mut().insert(&0_u64)?;

        Ok(VecStorage {
            phantom_data: PhantomData,
            storage,
            storage_index,
            indexes: vec![],
            len: 0,
            capacity: 0,
        })
    }

    pub fn storage_index(&self) -> DbIndex {
        self.storage_index.clone()
    }

    fn index_offset(index: u64) -> u64 {
        Self::offset::<DbIndex>(index)
    }

    fn offset<V: SerializeFixedSized>(index: u64) -> u64 {
        u64::serialized_size() + V::serialized_size() * index
    }

    fn grow<V: SerializeFixedSized>(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        if size >= self.capacity() {
            self.reallocate::<V>(size)?;
        }

        let bytes = value.serialize().repeat((size - self.len()) as usize);

        self.storage.borrow_mut().insert_bytes_at(
            &self.storage_index,
            Self::offset::<V>(self.len),
            &bytes,
        )?;

        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &size)?;
        self.len = size;

        Ok(())
    }

    fn grow_dynamic(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        if size >= self.capacity() {
            self.reallocate_indexes(size)?;
        }

        let mut new_indexes = Vec::<DbIndex>::new();
        new_indexes.reserve((size - self.len()) as usize);

        for _ in self.len()..size {
            let index = self.storage.borrow_mut().insert(value)?;
            new_indexes.push(index);
        }

        self.indexes.extend(&new_indexes);

        let bytes = new_indexes.serialize();

        self.storage.borrow_mut().insert_bytes_at(
            &self.storage_index,
            Self::index_offset(self.len),
            &bytes[8..],
        )?;

        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &size)?;
        self.len = size;

        Ok(())
    }

    fn reallocate_indexes(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.indexes.shrink_to_fit();
        self.reallocate::<DbIndex>(new_capacity)
    }

    fn reallocate<V: SerializeFixedSized>(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.capacity = new_capacity;
        self.storage.borrow_mut().resize_value(
            &self.storage_index,
            u64::serialized_size() + V::serialized_size() * self.capacity,
        )?;

        Ok(())
    }

    fn shrink<V: SerializeFixedSized>(&mut self, size: u64) -> Result<(), DbError> {
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &size)?;
        self.len = size;

        Ok(())
    }

    fn shrink_dynamic(&mut self, size: u64) -> Result<(), DbError> {
        for i in size..self.len() {
            let index = self.indexes[i as usize];
            self.storage.borrow_mut().remove(&index)?;
        }

        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &size)?;
        self.indexes.resize(size as usize, DbIndex::default());
        self.len = size;

        Ok(())
    }

    fn validate_index(&self, index: u64) -> Result<(), DbError> {
        if self.len() <= index {
            return Err(DbError::from(format!(
                "VecStorage error: index ({}) out of bounds ({})",
                index, self.len
            )));
        }

        Ok(())
    }
}
