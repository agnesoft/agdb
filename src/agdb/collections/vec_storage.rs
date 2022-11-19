use super::vec::vec_storage_iterator::VecStorageIterator;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeDynamicSized;
use crate::utilities::serialize::SerializeFixedSized;
use crate::DbError;
use crate::DbIndex;
use std::cell::RefCell;
use std::cmp::max;
use std::marker::PhantomData;
use std::rc::Rc;

pub trait VecDynamicSized<T, Data>
where
    T: SerializeDynamicSized,
    Data: Storage,
{
    fn iter(&self) -> VecStorageIterator<T, Data>;
    fn push(&mut self, value: &T) -> Result<(), DbError>;
    fn remove(&mut self, index: u64) -> Result<(), DbError>;
    fn reserve(&mut self, capacity: u64) -> Result<(), DbError>;
    fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn to_vec(&self) -> Result<Vec<T>, DbError>;
    fn value(&self, index: u64) -> Result<T, DbError>;
}
pub trait VecFixedSized<T, Data>
where
    T: SerializeFixedSized,
    Data: Storage,
{
    fn iter(&self) -> VecStorageIterator<T, Data>;
    fn push(&mut self, value: &T) -> Result<(), DbError>;
    fn remove(&mut self, index: u64) -> Result<(), DbError>;
    fn reserve(&mut self, capacity: u64) -> Result<(), DbError>;
    fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn shrink_to_fit(&mut self) -> Result<(), DbError>;
    fn to_vec(&self) -> Result<Vec<T>, DbError>;
    fn value(&self, index: u64) -> Result<T, DbError>;
}

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

    pub fn new(data: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        todo!()
    }

    pub fn from_storage(data: Rc<RefCell<Data>>, index: &DbIndex) -> Result<Self, DbError> {
        todo!()
    }

    pub fn storage_index(&self) -> DbIndex {
        self.storage_index.clone()
    }

    fn dynamic_offset(index: u64) -> u64 {
        u64::serialized_size() + DbIndex::serialized_size() * index
    }

    fn grow_dynamic(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        if size >= self.capacity() {
            self.reallocate_dynamic(size)?;
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
            Self::dynamic_offset(self.len),
            &bytes[8..],
        )?;

        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &size)?;
        self.len = size;

        Ok(())
    }

    fn reallocate_dynamic(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.capacity = self.storage.borrow_mut().resize_value(
            &self.storage_index,
            u64::serialized_size() + DbIndex::serialized_size() * max(64, new_capacity),
        )?;

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

impl<T, Data> VecDynamicSized<T, Data> for VecStorage<T, Data>
where
    T: SerializeDynamicSized,
    Data: Storage,
{
    fn iter(&self) -> VecStorageIterator<T, Data> {
        todo!()
    }

    fn push(&mut self, value: &T) -> Result<(), DbError> {
        self.storage.borrow_mut().transaction();

        if self.len() == self.capacity() {
            self.reallocate_dynamic(self.capacity * 2)?;
        }

        let index = self.storage.borrow_mut().insert(value)?;
        self.indexes.push(index);
        self.storage.borrow_mut().insert_at(
            &self.storage_index,
            Self::dynamic_offset(self.len),
            &index,
        )?;

        self.len += 1;

        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &self.len)?;

        self.storage.borrow_mut().commit()
    }

    fn remove(&mut self, index: u64) -> Result<(), DbError> {
        self.validate_index(index)?;

        self.storage.borrow_mut().transaction();
        let value_index = self.indexes[index as usize];
        self.storage.borrow_mut().remove(&value_index)?;
        self.indexes.remove(index as usize);
        let offset_from = Self::dynamic_offset(index + 1);
        self.storage.borrow_mut().move_at(
            &self.storage_index,
            Self::dynamic_offset(index + 1),
            Self::dynamic_offset(index),
            Self::dynamic_offset(self.len) - offset_from,
        )?;
        self.len -= 1;
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &self.len)?;

        self.storage.borrow_mut().commit()
    }

    fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if capacity <= self.capacity() {
            return Ok(());
        }

        self.reallocate_dynamic(capacity)
    }

    fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        if self.len() == size {
            return Ok(());
        }

        self.storage.borrow_mut().transaction();

        if size < self.len() {
            self.shrink_dynamic(size)?;
        } else if self.len() < size {
            self.grow_dynamic(size, value)?;
        }

        self.storage.borrow_mut().commit()
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.validate_index(index)?;

        let old_index = self.indexes[index as usize];

        self.storage.borrow_mut().transaction();
        self.storage.borrow_mut().remove(&old_index)?;
        let new_index = self.storage.borrow_mut().insert(value)?;
        self.indexes[index as usize] = new_index;
        self.storage.borrow_mut().insert_at(
            &self.storage_index,
            Self::dynamic_offset(index),
            &new_index,
        )?;

        self.storage.borrow_mut().commit()
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.reallocate_dynamic(self.len())
    }

    fn to_vec(&self) -> Result<Vec<T>, DbError> {
        let mut vec = Vec::<T>::new();
        vec.reserve(self.len() as usize);

        for index in &self.indexes {
            vec.push(self.storage.borrow_mut().value(index)?);
        }

        Ok(vec)
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        self.validate_index(index)?;
        self.storage
            .borrow_mut()
            .value(&self.indexes[index as usize])
    }
}

impl<T, Data> VecFixedSized<T, Data> for VecStorage<T, Data>
where
    T: SerializeFixedSized,
    Data: Storage,
{
    fn iter(&self) -> VecStorageIterator<T, Data> {
        todo!()
    }

    fn push(&mut self, value: &T) -> Result<(), DbError> {
        todo!()
    }

    fn remove(&mut self, index: u64) -> Result<(), DbError> {
        todo!()
    }

    fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        todo!()
    }

    fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        todo!()
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        todo!()
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        todo!()
    }

    fn to_vec(&self) -> Result<Vec<T>, DbError> {
        todo!()
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        todo!()
    }
}

#[cfg(test)]
mod tests_fixed_size {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1_i64).unwrap();
        vec.push(&3_i64).unwrap();
        vec.push(&5_i64).unwrap();

        assert_eq!(vec.iter().collect::<Vec<i64>>(), vec![1_i64, 3_i64, 5_i64]);
    }

    #[test]
    fn is_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();

        assert!(vec.is_empty());

        vec.push(&1).unwrap();

        assert!(!vec.is_empty());
    }

    #[test]
    fn len() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();

        assert_eq!(vec.len(), 0);

        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.len(), 3)
    }

    #[test]
    fn min_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.capacity(), 64);
    }

    #[test]
    fn push() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(&vec.storage_index()),
            Ok(vec![1_i64, 3_i64, 5_i64])
        );
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(1).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1, 5]));
    }

    #[test]
    fn remove_at_end() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(2).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1, 3]));
    }

    #[test]
    fn remove_index_out_of_bounds() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();

        assert_eq!(vec.remove(0), Err(DbError::from("index out of bounds")));
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
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

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
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

        let mut vec = VecStorage::<i64>::new(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(6, &0).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(&vec.storage_index()),
            Ok(vec![1_i64, 3_i64, 5_i64, 0, 0, 0])
        );
    }

    #[test]
    fn resize_over_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(100, &0).unwrap();

        let mut expected = vec![0_i64; 100];
        expected[0] = 1;
        expected[1] = 3;
        expected[2] = 5;

        assert_eq!(vec.len(), 100);
        assert_eq!(vec.capacity(), 100);

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(&vec.storage_index()),
            Ok(expected)
        );
    }

    #[test]
    fn resize_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(3, &0).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(&vec.storage_index()),
            Ok(vec![1_i64, 3_i64, 5_i64])
        );
    }

    #[test]
    fn resize_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(1, &0).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(&vec.storage_index()),
            Ok(vec![1_i64])
        );
    }

    #[test]
    fn set_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.set_value(1, &10).unwrap();

        assert_eq!(vec.value(0), Ok(1));
        assert_eq!(vec.value(1), Ok(10));
        assert_eq!(vec.value(2), Ok(5));
    }

    #[test]
    fn set_value_out_of_bounds() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();

        assert_eq!(
            vec.set_value(0, &10),
            Err(DbError::from("index out of bounds"))
        );
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.capacity(), 64);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 3);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn shrink_to_fit_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();

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

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
    }

    #[test]
    fn try_from_storage_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let index;

        {
            let mut vec = VecStorage::<i64>::new(storage.clone()).unwrap();
            vec.push(&1).unwrap();
            vec.push(&3).unwrap();
            vec.push(&5).unwrap();
            index = vec.storage_index();
        }

        let vec = VecStorage::<i64>::from_storage(storage, &index).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
    }

    #[test]
    fn try_from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        assert_eq!(
            VecStorage::<i64>::from_storage(storage, &DbIndex::from(1_u64))
                .err()
                .unwrap(),
            DbError::from("index '1' not found")
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.value(0), Ok(1));
        assert_eq!(vec.value(1), Ok(3));
        assert_eq!(vec.value(2), Ok(5));
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let vec = VecStorage::<i64>::new(storage).unwrap();

        assert_eq!(vec.value(0), Err(DbError::from("index out of bounds")));
    }
}

#[cfg(test)]
mod tests_dynamic_size {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

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
            .value::<Vec<DbIndex>>(&vec.storage_index())
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

        assert_eq!(vec.remove(0), Err(DbError::from("index out of bounds")));
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
            Err(DbError::from("index out of bounds"))
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
    fn try_from_storage_index() {
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
    fn try_from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        assert_eq!(
            VecStorage::<String>::from_storage(storage, &DbIndex::from(1_u64))
                .err()
                .unwrap(),
            DbError::from("index '1' not found")
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

        assert_eq!(vec.value(0), Err(DbError::from("index out of bounds")));
    }
}
