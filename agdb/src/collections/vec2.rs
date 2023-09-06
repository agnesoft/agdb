use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use std::marker::PhantomData;

pub trait VecValue: Sized {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError>;
    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError>;
    fn remove<S: Storage>(storage: &mut S, _bytes: &[u8]) -> Result<(), DbError>;
    fn storage_len() -> u64;
}

pub struct DbVec<T, S = FileStorage>
where
    T: VecValue,
    S: Storage,
{
    capacity: u64,
    len: u64,
    storage_index: StorageIndex,
    phantom_data: PhantomData<(T, S)>,
}

pub struct VecIterator<'a, T, S>
where
    T: VecValue,
    S: Storage,
{
    pub index: u64,
    pub vec: &'a DbVec<T, S>,
    pub storage: &'a S,
}

pub struct DbVecImpl<'a, T, S>
where
    T: VecValue,
    S: Storage,
{
    vec: &'a DbVec<T, S>,
    storage: &'a S,
}

pub struct DbVecImplMut<'a, T, S>
where
    T: VecValue,
    S: Storage,
{
    vec: &'a mut DbVec<T, S>,
    storage: &'a mut S,
}

impl<T, S> DbVec<T, S>
where
    T: VecValue,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn from_storage(storage: &S, storage_index: StorageIndex) -> Result<Self, DbError> {
        let len = storage.value::<u64>(storage_index)?;
        let data_len = storage.value_size(storage_index)?;
        let capacity = data_len / T::storage_len();

        Ok(Self {
            capacity,
            len,
            storage_index,
            phantom_data: PhantomData,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn new(storage: &mut S) -> Result<Self, DbError> {
        let storage_index = storage.insert(&0_u64)?;

        Ok(Self {
            capacity: 0,
            len: 0,
            storage_index,
            phantom_data: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    pub fn read<'a>(&'a self, storage: &'a S) -> DbVecImpl<'a, T, S> {
        DbVecImpl { vec: self, storage }
    }

    pub fn write<'a>(&'a mut self, storage: &'a mut S) -> DbVecImplMut<'a, T, S> {
        DbVecImplMut { vec: self, storage }
    }

    fn iter<'a>(&'a self, storage: &'a S) -> VecIterator<'a, T, S> {
        VecIterator {
            index: 0,
            vec: self,
            storage,
        }
    }

    fn offset(index: u64) -> u64 {
        u64::serialized_size_static() + T::storage_len() * index
    }

    fn reallocate(&mut self, storage: &mut S, capacity: u64) -> Result<(), DbError> {
        storage.resize_value(
            self.storage_index,
            self.len().serialized_size() + T::storage_len() * capacity,
        )?;
        self.capacity = capacity;

        Ok(())
    }

    fn remove(&mut self, storage: &mut S, index: u64) -> Result<T, DbError> {
        let offset_from = Self::offset(index + 1);
        let offset_to = Self::offset(index);
        let move_len = T::storage_len() * (self.len() - index);
        let bytes = storage.value_as_bytes_at_size(
            self.storage_index,
            Self::offset(index),
            T::storage_len(),
        )?;
        let old_value = T::load(storage, &bytes)?;
        let id = storage.transaction();
        T::remove(storage, &bytes)?;
        storage.move_at(self.storage_index, offset_from, offset_to, move_len)?;
        storage.insert_at(self.storage_index, 0, &(self.len() - 1))?;
        storage.commit(id)?;

        Ok(old_value)
    }

    fn replace(&mut self, storage: &mut S, index: u64, value: &T) -> Result<T, DbError> {
        let old_bytes = storage.value_as_bytes_at_size(
            self.storage_index,
            Self::offset(index),
            T::storage_len(),
        )?;
        let old_value = T::load(storage, &old_bytes)?;
        let id = storage.transaction();
        T::remove(storage, &old_bytes)?;
        let bytes = value.store(storage)?;
        storage.insert_bytes_at(self.storage_index, Self::offset(index), &bytes)?;
        storage.commit(id)?;

        Ok(old_value)
    }

    fn resize(&mut self, storage: &mut S, new_len: u64, value: &T) -> Result<(), DbError> {
        let id = storage.transaction();

        for index in self.len()..new_len {
            let bytes = value.store(storage)?;
            storage.insert_bytes_at(self.storage_index, Self::offset(index), &bytes)?;
        }

        for index in new_len..self.len() {
            let old_bytes = storage.value_as_bytes_at_size(
                self.storage_index,
                Self::offset(index),
                T::storage_len(),
            )?;
            T::remove(storage, &old_bytes)?;
        }

        storage.insert_at(self.storage_index, 0, &new_len)?;
        storage.commit(id)?;
        self.len = new_len;

        Ok(())
    }

    fn swap(&mut self, storage: &mut S, index: u64, other: u64) -> Result<(), DbError> {
        let offset_from = Self::offset(other);
        let offset_to = Self::offset(index);
        let size = T::storage_len();
        let bytes = storage.value_as_bytes_at_size(
            self.storage_index,
            Self::offset(index),
            T::storage_len(),
        )?;
        let id = storage.transaction();
        storage.move_at(self.storage_index, offset_from, offset_to, size)?;
        storage.insert_bytes_at(self.storage_index, Self::offset(other), &bytes)?;
        storage.commit(id)
    }

    fn value(&self, storage: &S, index: u64) -> Result<T, DbError> {
        self.validate_index(index)?;
        let bytes = storage.value_as_bytes_at_size(
            self.storage_index,
            Self::offset(index),
            T::storage_len(),
        )?;
        Ok(T::load(storage, &bytes)?)
    }

    fn validate_index(&self, index: u64) -> Result<(), DbError> {
        if self.len() <= index {
            return Err(DbError::from(format!(
                "Index ({index}) out of bounds ({})",
                self.len()
            )));
        }

        Ok(())
    }
}

impl<'a, T, S> DbVecImpl<'a, T, S>
where
    T: VecValue,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.vec.capacity()
    }

    pub fn len(&self) -> u64 {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> VecIterator<T, S> {
        self.vec.iter(self.storage)
    }

    pub fn value(&self, index: u64) -> Result<T, DbError> {
        self.vec.value(self.storage, index)
    }
}

impl<'a, T, S> DbVecImplMut<'a, T, S>
where
    T: VecValue,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.vec.capacity()
    }

    pub fn len(&self) -> u64 {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> VecIterator<T, S> {
        self.vec.iter(self.storage)
    }

    pub fn value(&self, index: u64) -> Result<T, DbError> {
        self.vec.value(self.storage, index)
    }

    pub fn push(&mut self, value: &T) -> Result<(), DbError> {
        if self.len() == self.capacity() {
            self.vec.reallocate(
                self.storage,
                std::cmp::max(64, self.capacity() + self.capacity() / 2),
            )?;
        }

        self.vec.resize(self.storage, self.len() + 1, value)
    }

    pub fn remove(&mut self, index: u64) -> Result<T, DbError> {
        self.vec.validate_index(index)?;
        self.vec.remove(self.storage, index)
    }

    pub fn replace(&mut self, index: u64, value: &T) -> Result<T, DbError> {
        self.vec.validate_index(index)?;
        self.vec.replace(self.storage, index, value)
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if self.capacity() < capacity {
            self.vec.reallocate(self.storage, capacity)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, new_len: u64, value: &T) -> Result<(), DbError> {
        if self.capacity() < new_len {
            self.vec.reallocate(self.storage, new_len)?;
        }

        self.vec.resize(self.storage, new_len, value)
    }

    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.vec.reallocate(self.storage, self.len())
    }

    pub fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError> {
        if index == other {
            return Ok(());
        }

        self.vec.validate_index(index)?;
        self.vec.validate_index(other)?;
        self.vec.swap(self.storage, index, other)
    }
}

impl<'a, T, S> Iterator for VecIterator<'a, T, S>
where
    T: Clone + VecValue,
    S: Storage,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.storage, self.index).ok();
        self.index += 1;

        value
    }
}

impl VecValue for u64 {
    fn store<S: Storage>(&self, _storage: &mut S) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<S: Storage>(_storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove<S: Storage>(_storage: &mut S, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

impl VecValue for i64 {
    fn store<S: Storage>(&self, _storage: &mut S) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<S: Storage>(_storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove<S: Storage>(_storage: &mut S, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

impl VecValue for String {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError> {
        let index = storage.insert(self)?;
        Ok(index.serialize())
    }

    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        let index = StorageIndex::deserialize(bytes)?;
        storage.value(index)
    }

    fn remove<S: Storage>(storage: &mut S, bytes: &[u8]) -> Result<(), DbError> {
        let index = StorageIndex::deserialize(bytes)?;
        storage.remove(index)
    }

    fn storage_len() -> u64 {
        StorageIndex::serialized_size_static()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn from_storage_index() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let index;

        {
            let mut vec = DbVec::<String>::new(&mut storage).unwrap();
            let mut write_vec = vec.write(&mut storage);
            write_vec.push(&"Hello".to_string()).unwrap();
            write_vec.push(&", ".to_string()).unwrap();
            write_vec.push(&"World".to_string()).unwrap();
            write_vec.push(&"!".to_string()).unwrap();
            index = vec.storage_index();
        }

        let vec = DbVec::<String>::from_storage(&storage, index).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ]
        );
    }

    #[test]
    fn from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = FileStorage::new(test_file.file_name()).unwrap();

        assert_eq!(
            DbVec::<String>::from_storage(&storage, StorageIndex::from(1_u64))
                .err()
                .unwrap(),
            DbError::from("FileStorage error: index (1) not found")
        );
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec!["Hello", ", ", "World", "!"]
        );
    }

    #[test]
    fn is_empty() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();

        assert!(vec.is_empty());

        vec.write(&mut storage)
            .push(&"Hello, World!".to_string())
            .unwrap();

        assert!(!vec.is_empty());
    }

    #[test]
    fn len() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();

        assert_eq!(vec.len(), 0);

        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        assert_eq!(vec.len(), 4)
    }

    #[test]
    fn min_capacity() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        assert_eq!(vec.capacity(), 64);
    }

    #[test]
    fn push() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        let indexes = storage
            .value::<Vec<StorageIndex>>(vec.storage_index())
            .unwrap();

        let mut values = Vec::<String>::new();

        for index in indexes {
            values.push(storage.value::<String>(index).unwrap());
        }

        assert_eq!(
            values,
            vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ]
        );
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.remove(1).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec!["Hello".to_string(), "World".to_string(), "!".to_string()]
        );
    }

    #[test]
    fn remove_at_end() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.remove(2).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec!["Hello".to_string(), ", ".to_string(), "!".to_string(),]
        );
    }

    #[test]
    fn remove_index_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);

        assert_eq!(
            write_vec.remove(0),
            Err(DbError::from("Index (0) out of bounds (0)"))
        );
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        assert_eq!(vec.capacity(), 0);

        vec.write(&mut storage).reserve(20).unwrap();

        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.reserve(20).unwrap();
        write_vec.reserve(10).unwrap();

        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn resize_larger() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.resize(6, &" ".to_string()).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string(),
                " ".to_string(),
                " ".to_string(),
            ]
        );
    }

    #[test]
    fn resize_over_capacity() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.resize(100, &" ".to_string()).unwrap();

        let mut expected = Vec::<String>::new();
        expected.resize(100, " ".to_string());
        expected[0] = "Hello".to_string();
        expected[1] = ", ".to_string();
        expected[2] = "World".to_string();
        expected[3] = "!".to_string();

        assert_eq!(vec.len(), 100);
        assert_eq!(vec.capacity(), 100);

        assert_eq!(vec.read(&storage).iter().collect::<Vec<String>>(), expected);
    }

    #[test]
    fn resize_same() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.resize(4, &String::default()).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ]
        );
    }

    #[test]
    fn resize_smaller() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.resize(3, &String::default()).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec!["Hello".to_string(), ", ".to_string(), "World".to_string()]
        );
    }

    #[test]
    fn replace() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.replace(1, &" ".to_string()).unwrap();

        let read_vec = vec.read(&storage);

        assert_eq!(read_vec.value(0), Ok("Hello".to_string()));
        assert_eq!(read_vec.value(1), Ok(" ".to_string()));
        assert_eq!(read_vec.value(2), Ok("World".to_string()));
        assert_eq!(read_vec.value(3), Ok("!".to_string()));
    }

    #[test]
    fn replace_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();

        assert_eq!(
            vec.write(&mut storage).replace(0, &"".to_string()),
            Err(DbError::from("Index (0) out of bounds (0)"))
        );
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        assert_eq!(write_vec.capacity(), 64);

        write_vec.shrink_to_fit().unwrap();

        assert_eq!(write_vec.capacity(), 4);

        write_vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 4);
    }

    #[test]
    fn shrink_to_fit_empty() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.write(&mut storage).shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn swap() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        write_vec.swap(0, 2).unwrap();

        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec![
                "World".to_string(),
                ", ".to_string(),
                "Hello".to_string(),
                "!".to_string()
            ]
        );
    }
    #[test]
    fn swap_self() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();
        write_vec.swap(1, 1).unwrap();
        assert_eq!(
            vec.read(&storage).iter().collect::<Vec<String>>(),
            vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ]
        );
    }
    #[test]
    fn swap_invalid() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();
        assert_eq!(
            write_vec.swap(1, 10),
            Err(DbError::from("Index (10) out of bounds (4)"))
        );
        assert_eq!(
            write_vec.swap(10, 1),
            Err(DbError::from("Index (10) out of bounds (4)"))
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let mut vec = DbVec::<String>::new(&mut storage).unwrap();
        let mut write_vec = vec.write(&mut storage);
        write_vec.push(&"Hello".to_string()).unwrap();
        write_vec.push(&", ".to_string()).unwrap();
        write_vec.push(&"World".to_string()).unwrap();
        write_vec.push(&"!".to_string()).unwrap();

        let read_vec = vec.read(&storage);

        assert_eq!(read_vec.value(0), Ok("Hello".to_string()));
        assert_eq!(read_vec.value(1), Ok(", ".to_string()));
        assert_eq!(read_vec.value(2), Ok("World".to_string()));
        assert_eq!(read_vec.value(3), Ok("!".to_string()));
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let vec = DbVec::<String>::new(&mut storage).unwrap();

        assert_eq!(
            vec.read(&storage).value(0),
            Err(DbError::from("Index (0) out of bounds (0)"))
        );
    }
}
