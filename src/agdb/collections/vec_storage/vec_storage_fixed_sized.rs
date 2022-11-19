use super::VecStorage;
use crate::collections::vec::vec_fixed_sized::VecFixedSized;
use crate::collections::vec::vec_fixed_sized_iterator::VecFixedSizedIterator;
use crate::storage::Storage;
use crate::DbError;
use crate::DbIndex;
use std::cell::RefCell;
use std::cmp::max;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::utilities::serialize::SerializeFixedSized;

impl<T, Data> VecFixedSized<T, Data> for VecStorage<T, Data>
where
    T: SerializeFixedSized,
    Data: Storage,
{
    fn from_storage(storage: Rc<RefCell<Data>>, index: &DbIndex) -> Result<Self, DbError> {
        let len = storage.borrow_mut().value::<u64>(&index)?;
        let capacity = (storage.borrow_mut().value_size(&index)? - u64::serialized_size())
            / T::serialized_size();

        Ok(VecStorage {
            phantom_data: PhantomData,
            storage,
            storage_index: index.clone(),
            indexes: vec![],
            len,
            capacity,
        })
    }

    fn iter(&self) -> VecFixedSizedIterator<T, Data> {
        VecFixedSizedIterator::<T, Data> {
            index: 0,
            vec: self,
            phantom_data: PhantomData,
        }
    }

    fn push(&mut self, value: &T) -> Result<(), DbError> {
        self.storage.borrow_mut().transaction();

        if self.len() == self.capacity() {
            self.reallocate::<T>(max(64, self.capacity * 2))?;
        }

        self.storage.borrow_mut().insert_at(
            &self.storage_index,
            Self::offset::<T>(self.len),
            value,
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
        let offset_from = Self::offset::<T>(index + 1);
        self.storage.borrow_mut().move_at(
            &self.storage_index,
            offset_from,
            Self::offset::<T>(index),
            T::serialized_size(),
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

        self.reallocate::<T>(capacity)
    }

    fn resize(&mut self, size: u64, value: &T) -> Result<(), DbError> {
        if self.len() == size {
            return Ok(());
        }

        self.storage.borrow_mut().transaction();

        if size < self.len() {
            self.shrink::<T>(size)?;
        } else if self.len() < size {
            self.grow::<T>(size, value)?;
        }

        self.storage.borrow_mut().commit()
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.validate_index(index)?;

        self.storage.borrow_mut().insert_at(
            &self.storage_index,
            Self::offset::<T>(index),
            value,
        )?;

        Ok(())
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.reallocate::<T>(self.len())
    }

    fn to_vec(&self) -> Result<Vec<T>, DbError> {
        self.storage
            .borrow_mut()
            .value::<Vec<T>>(&self.storage_index)
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        self.validate_index(index)?;

        self.storage
            .borrow_mut()
            .value_at::<T>(&self.storage_index, Self::offset::<T>(index))
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
    fn from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        assert_eq!(
            VecStorage::<i64>::from_storage(storage, &DbIndex::from(1_u64))
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

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        assert_eq!(vec.capacity(), 0);

        vec.reserve(70).unwrap();

        assert_eq!(vec.capacity(), 70);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut vec = VecStorage::<i64>::new(storage).unwrap();
        vec.reserve(70).unwrap();

        assert_eq!(vec.capacity(), 70);

        vec.reserve(10).unwrap();

        assert_eq!(vec.capacity(), 70);
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

        assert_eq!(
            vec.value(0),
            Err(DbError::from(
                "VecStorage error: index (0) out of bounds (0)"
            ))
        );
    }
}
