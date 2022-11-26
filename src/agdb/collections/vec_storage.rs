pub mod vec_storage_iterator;

use self::vec_storage_iterator::VecStorageIterator;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
use crate::DbError;
use std::cell::RefCell;
use std::cmp::max;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct VecStorage<T, Data = FileStorage>
where
    T: StorageValue,
    Data: Storage,
{
    phantom_data: PhantomData<T>,
    storage: Rc<RefCell<Data>>,
    storage_index: StorageIndex,
    len: u64,
    capacity: u64,
}

#[allow(dead_code)]
impl<T, Data> VecStorage<T, Data>
where
    T: StorageValue,
    Data: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: &StorageIndex) -> Result<Self, DbError> {
        let len = storage.borrow_mut().value::<u64>(index)?;
        let capacity = (storage.borrow_mut().value_size(index)? - u64::static_serialized_size())
            / T::storage_len();

        Ok(VecStorage {
            phantom_data: PhantomData,
            storage,
            storage_index: *index,
            len,
            capacity,
        })
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> VecStorageIterator<T, Data> {
        VecStorageIterator::<T, Data> {
            index: 0,
            vec: self,
            phantom_data: PhantomData,
        }
    }

    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let storage_index = storage.borrow_mut().insert(&0_u64)?;

        Ok(VecStorage {
            phantom_data: PhantomData,
            storage,
            storage_index,
            len: 0,
            capacity: 0,
        })
    }

    pub fn push(&mut self, value: &T) -> Result<(), DbError> {
        self.storage.borrow_mut().transaction();

        if self.len() == self.capacity() {
            self.reallocate(max(64, self.capacity * 2))?;
        }

        self.set_value_bytes(self.len(), value)?;
        self.set_len(self.len() + 1)?;
        self.storage.borrow_mut().commit()
    }

    pub fn remove(&mut self, index: u64) -> Result<(), DbError> {
        self.validate_index(index)?;

        self.storage.borrow_mut().transaction();
        self.remove_value_bytes(index)?;
        self.move_left(index)?;
        self.set_len(self.len() - 1)?;
        self.storage.borrow_mut().commit()
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if capacity <= self.capacity() {
            return Ok(());
        }

        self.reallocate(capacity)
    }

    pub fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        self.storage.borrow_mut().transaction();

        match size.cmp(&self.len()) {
            std::cmp::Ordering::Less => self.shrink(size)?,
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Greater => self.grow(size, value)?,
        }

        self.storage.borrow_mut().commit()
    }

    pub fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.validate_index(index)?;

        self.storage.borrow_mut().transaction();
        self.remove_value_bytes(index)?;
        self.set_value_bytes(index, value)?;
        self.storage.borrow_mut().commit()
    }

    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.reallocate(self.len())
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    pub fn to_vec(&self) -> Result<Vec<T>, DbError> {
        let mut vector = Vec::<T>::new();
        vector.reserve(self.len() as usize);

        for index in 0..self.len() {
            vector.push(self.load_value(index)?);
        }

        Ok(vector)
    }

    pub fn value(&self, index: u64) -> Result<T, DbError> {
        self.validate_index(index)?;
        self.load_value(index)
    }

    fn offset(index: u64) -> u64 {
        u64::static_serialized_size() + T::storage_len() * index
    }

    fn grow(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        if size >= self.capacity() {
            self.reallocate(size)?;
        }

        for index in self.len()..size {
            self.set_value_bytes(index, value)?;
        }

        self.set_len(size)
    }

    fn load_value(&self, index: u64) -> Result<T, DbError> {
        let bytes = self.value_bytes(index)?;
        T::load(&*self.storage.borrow_mut(), &bytes)
    }

    fn move_left(&mut self, index: u64) -> Result<(), DbError> {
        let offset_from = Self::offset(index + 1);
        let offset_to = Self::offset(index);
        let move_len = T::storage_len() * (self.len() - index);
        self.storage
            .borrow_mut()
            .move_at(&self.storage_index, offset_from, offset_to, move_len)
    }

    fn reallocate(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.capacity = new_capacity;
        self.storage.borrow_mut().resize_value(
            &self.storage_index,
            self.len().serialized_size() + T::storage_len() * self.capacity,
        )
    }

    fn remove_value_bytes(&mut self, index: u64) -> Result<(), DbError> {
        let bytes = self.value_bytes(index)?;
        T::remove(&mut *self.storage.borrow_mut(), &bytes)
    }

    fn set_len(&mut self, new_len: u64) -> Result<(), DbError> {
        self.len = new_len;
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &self.len)
    }

    fn set_value_bytes(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        let bytes = value.store(&mut *self.storage.borrow_mut())?;
        self.storage
            .borrow_mut()
            .insert_bytes_at(&self.storage_index, Self::offset(index), &bytes)
    }

    fn shrink(&mut self, size: u64) -> Result<(), DbError> {
        for index in size..self.len() {
            self.remove_value_bytes(index)?;
        }

        self.set_len(size)
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

    fn value_bytes(&self, index: u64) -> Result<Vec<u8>, DbError> {
        self.storage.borrow_mut().value_as_bytes_at_size(
            &self.storage_index,
            Self::offset(index),
            T::storage_len(),
        )
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
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let index;

        {
            let mut vec = VecStorage::<String>::new(storage.clone()).unwrap();
            vec.push(&"Hello".to_string()).unwrap();
            vec.push(&", ".to_string()).unwrap();
            vec.push(&"World".to_string()).unwrap();
            vec.push(&"!".to_string()).unwrap();
            index = vec.storage_index();
        }

        let vec = VecStorage::<String>::from_storage(storage, &index).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ])
        );
    }

    #[test]
    fn from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        assert_eq!(
            VecStorage::<String>::from_storage(storage, &StorageIndex::from(1_u64))
                .err()
                .unwrap(),
            DbError::from("FileStorage error: index (1) not found")
        );
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        assert_eq!(
            vec.iter().collect::<Vec<String>>(),
            vec!["Hello", ", ", "World", "!"]
        );
    }

    #[test]
    fn is_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();

        assert!(vec.is_empty());

        vec.push(&"Hello, World!".to_string()).unwrap();

        assert!(!vec.is_empty());
    }

    #[test]
    fn len() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();

        assert_eq!(vec.len(), 0);

        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        assert_eq!(vec.len(), 4)
    }

    #[test]
    fn min_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        assert_eq!(vec.capacity(), 64);
    }

    #[test]
    fn push() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage.clone()).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        let indexes = storage
            .borrow_mut()
            .value::<Vec<StorageIndex>>(&vec.storage_index())
            .unwrap();

        let mut values = Vec::<String>::new();

        for index in indexes {
            values.push(storage.borrow_mut().value::<String>(&index).unwrap());
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
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.remove(1).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec![
                "Hello".to_string(),
                "World".to_string(),
                "!".to_string()
            ])
        );
    }

    #[test]
    fn remove_at_end() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.remove(2).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec!["Hello".to_string(), ", ".to_string(), "!".to_string(),])
        );
    }

    #[test]
    fn remove_index_out_of_bounds() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();

        assert_eq!(
            vec.remove(0),
            Err(DbError::from(
                "VecStorage error: index (0) out of bounds (0)"
            ))
        );
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        assert_eq!(vec.capacity(), 0);

        vec.reserve(20).unwrap();

        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.reserve(20).unwrap();
        vec.reserve(10).unwrap();

        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn resize_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.resize(6, &" ".to_string()).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string(),
                " ".to_string(),
                " ".to_string(),
            ])
        );
    }

    #[test]
    fn resize_over_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.resize(100, &" ".to_string()).unwrap();

        let mut expected = Vec::<String>::new();
        expected.resize(100, " ".to_string());
        expected[0] = "Hello".to_string();
        expected[1] = ", ".to_string();
        expected[2] = "World".to_string();
        expected[3] = "!".to_string();

        assert_eq!(vec.len(), 100);
        assert_eq!(vec.capacity(), 100);

        assert_eq!(vec.to_vec(), Ok(expected));
    }

    #[test]
    fn resize_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.resize(4, &String::default()).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ])
        );
    }

    #[test]
    fn resize_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.resize(3, &String::default()).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string()
            ])
        );
    }

    #[test]
    fn set_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        vec.set_value(1, &" ".to_string()).unwrap();

        assert_eq!(vec.value(0), Ok("Hello".to_string()));
        assert_eq!(vec.value(1), Ok(" ".to_string()));
        assert_eq!(vec.value(2), Ok("World".to_string()));
        assert_eq!(vec.value(3), Ok("!".to_string()));
    }

    #[test]
    fn set_value_out_of_bounds() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();

        assert_eq!(
            vec.set_value(0, &"".to_string()),
            Err(DbError::from(
                "VecStorage error: index (0) out of bounds (0)"
            ))
        );
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        assert_eq!(vec.capacity(), 64);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 4);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 4);
    }

    #[test]
    fn shrink_to_fit_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn to_vec() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        assert_eq!(
            vec.to_vec(),
            Ok(vec![
                "Hello".to_string(),
                ", ".to_string(),
                "World".to_string(),
                "!".to_string()
            ])
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<String>::new(storage).unwrap();
        vec.push(&"Hello".to_string()).unwrap();
        vec.push(&", ".to_string()).unwrap();
        vec.push(&"World".to_string()).unwrap();
        vec.push(&"!".to_string()).unwrap();

        assert_eq!(vec.value(0), Ok("Hello".to_string()));
        assert_eq!(vec.value(1), Ok(", ".to_string()));
        assert_eq!(vec.value(2), Ok("World".to_string()));
        assert_eq!(vec.value(3), Ok("!".to_string()));
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let vec = VecStorage::<String>::new(storage).unwrap();

        assert_eq!(
            vec.value(0),
            Err(DbError::from(
                "VecStorage error: index (0) out of bounds (0)"
            ))
        );
    }
}
