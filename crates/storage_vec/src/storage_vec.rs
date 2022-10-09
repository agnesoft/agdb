use crate::vec_iterator::VecIterator;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;

pub struct StorageVec<T, Data = StorageFile>
where
    T: Serialize,
    Data: Storage,
{
    pub(crate) storage: std::rc::Rc<std::cell::RefCell<Data>>,
    pub(crate) storage_index: StorageIndex,
    pub(crate) len: u64,
    pub(crate) capacity: u64,
    pub(crate) phantom_data: std::marker::PhantomData<T>,
}

impl<T, Data> StorageVec<T, Data>
where
    T: Serialize,
    Data: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> VecIterator<T, Data> {
        VecIterator::<T, Data> {
            index: 0,
            vec: self,
            phantom_data: std::marker::PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn push(&mut self, value: &T) -> Result<(), DbError> {
        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.transaction();

        if self.len() == self.capacity {
            let current_capacity = self.capacity;
            Self::reallocate(
                &mut self.capacity,
                std::cmp::max(current_capacity * 2, 64),
                &mut ref_storage,
                &self.storage_index,
            )?;
        }

        ref_storage.insert_at(&self.storage_index, Self::value_offset(self.len()), value)?;
        self.len += 1;
        ref_storage.insert_at(&self.storage_index, 0, &self.len())?;
        ref_storage.commit()
    }

    pub fn remove(&mut self, index: u64) -> Result<(), DbError> {
        if self.len() <= index {
            return Err(DbError::from("index out of bounds"));
        }

        let offset_from = Self::value_offset(index + 1);
        let offset_to = Self::value_offset(index);
        let size = Self::value_offset(self.len()) - offset_from;

        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.transaction();
        ref_storage.move_at(&self.storage_index, offset_from, offset_to, size)?;
        self.len -= 1;
        ref_storage.insert_at(&self.storage_index, 0, &self.len())?;
        ref_storage.commit()
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if capacity <= self.capacity() {
            return Ok(());
        }

        let mut ref_storage = self.storage.borrow_mut();
        Self::reallocate(
            &mut self.capacity,
            capacity,
            &mut ref_storage,
            &self.storage_index,
        )
    }

    pub fn resize(&mut self, size: u64) -> Result<(), DbError> {
        if self.len() == size {
            return Ok(());
        }

        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.transaction();

        if size < self.len() {
            let offset = Self::value_offset(size);
            let byte_size = Self::value_offset(self.len()) - offset;
            ref_storage.insert_at(&self.storage_index, offset, &vec![0_u8; byte_size as usize])?;
        } else if self.capacity < size {
            Self::reallocate(
                &mut self.capacity,
                size,
                &mut ref_storage,
                &self.storage_index,
            )?;
        }

        self.len = size;
        ref_storage.insert_at(&self.storage_index, 0, &self.len())?;
        ref_storage.commit()
    }

    pub fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        if self.len() <= index {
            return Err(DbError::from("index out of bounds"));
        }

        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, Self::value_offset(index), value)
    }

    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        let current_len = self.len();
        let mut ref_storage = self.storage.borrow_mut();
        Self::reallocate(
            &mut self.capacity,
            current_len,
            &mut ref_storage,
            &self.storage_index,
        )
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index.clone()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_vec(&self) -> Result<Vec<T>, DbError> {
        self.storage.borrow_mut().value(&self.storage_index)
    }

    pub fn value(&self, index: u64) -> Result<T, DbError> {
        if self.len() <= index {
            return Err(DbError::from("index out of bounds"));
        }

        self.storage
            .borrow_mut()
            .value_at::<T>(&self.storage_index, Self::value_offset(index))
    }

    pub fn value_offset(index: u64) -> u64 {
        u64::serialized_size() + index * T::serialized_size()
    }

    pub(crate) fn capacity_from_bytes(len: u64) -> u64 {
        (len - u64::serialized_size()) / T::serialized_size()
    }

    fn reallocate(
        capacity: &mut u64,
        new_capacity: u64,
        storage: &mut std::cell::RefMut<Data>,
        index: &StorageIndex,
    ) -> Result<(), DbError> {
        *capacity = new_capacity;
        storage.resize_value(index, Self::value_offset(new_capacity))
    }
}
