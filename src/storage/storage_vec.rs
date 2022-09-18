use super::file_storage::FileStorage;
use super::serialize::Serialize;
use super::Storage;
use crate::db_error::DbError;

#[allow(dead_code)]
pub(crate) struct StorageVec<T: Serialize, S: Storage = FileStorage> {
    storage: std::rc::Rc<std::cell::RefCell<S>>,
    storage_index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Serialize, S: Storage> StorageVec<T, S> {
    pub(crate) fn capacity(&self) -> u64 {
        self.capacity
    }

    pub(crate) fn len(&self) -> u64 {
        self.size
    }

    pub(crate) fn push(&mut self, value: &T) -> Result<(), DbError> {
        if self.size == self.capacity {
            self.reallocate(std::cmp::max(self.capacity * 2, 64))?;
        }

        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.insert_at(self.storage_index, Self::value_offset(self.size), value)?;
        self.size += 1;
        ref_storage.insert_at(self.storage_index, 0, &self.size)?;

        Ok(())
    }

    pub(crate) fn remove(&mut self, index: u64) -> Result<(), DbError> {
        if self.size <= index {
            return Err(DbError::from("index out of bounds"));
        }

        let offset_from = Self::value_offset(index + 1);
        let offset_to = Self::value_offset(index);
        let size = Self::value_offset(self.size) - offset_from;
        self.storage
            .borrow_mut()
            .move_at(self.storage_index, offset_from, offset_to, size)?;
        self.size -= 1;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.size)
    }

    pub(crate) fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if capacity <= self.capacity {
            return Ok(());
        }

        self.reallocate(capacity)
    }

    pub(crate) fn resize(&mut self, size: u64) -> Result<(), DbError> {
        if self.size == size {
            return Ok(());
        }

        if size < self.size {
            let offset = Self::value_offset(size);
            let byte_size = Self::value_offset(self.size) - offset;
            self.storage.borrow_mut().insert_at(
                self.storage_index,
                offset,
                &vec![0_u8; byte_size as usize],
            )?;
        } else if self.capacity < size {
            self.reallocate(size)?;
        }

        self.size = size;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.size)
    }

    pub(crate) fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        if self.size <= index {
            return Err(DbError::from("index out of bounds"));
        }

        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, Self::value_offset(index), value)
    }

    pub(crate) fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.reallocate(self.size)
    }

    pub(crate) fn storage_index(&self) -> i64 {
        self.storage_index
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_vec(&mut self) -> Result<Vec<T>, DbError> {
        self.storage.borrow_mut().value(self.storage_index)
    }

    pub(crate) fn value(&mut self, index: u64) -> Result<T, DbError> {
        if self.size <= index {
            return Err(DbError::from("index out of bounds"));
        }

        self.storage
            .borrow_mut()
            .value_at::<T>(self.storage_index, Self::value_offset(index))
    }

    fn reallocate(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.capacity = new_capacity;
        self.storage
            .borrow_mut()
            .resize_value(self.storage_index, Self::value_offset(new_capacity))
    }

    fn value_offset(index: u64) -> u64 {
        u64::serialized_size() + index * T::serialized_size()
    }

    fn capacity_from_bytes(len: u64) -> u64 {
        (len - u64::serialized_size()) / T::serialized_size()
    }
}

impl<T: Serialize, S: Storage> TryFrom<std::rc::Rc<std::cell::RefCell<S>>> for StorageVec<T, S> {
    type Error = DbError;

    fn try_from(storage: std::rc::Rc<std::cell::RefCell<S>>) -> Result<Self, Self::Error> {
        let index = storage.borrow_mut().insert(&0_u64)?;

        Ok(Self {
            storage,
            storage_index: index,
            size: 0,
            capacity: 0,
            phantom_data: std::marker::PhantomData::<T>,
        })
    }
}

impl<T: Serialize, S: Storage> TryFrom<(std::rc::Rc<std::cell::RefCell<S>>, i64)>
    for StorageVec<T, S>
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (std::rc::Rc<std::cell::RefCell<S>>, i64),
    ) -> Result<Self, Self::Error> {
        let byte_size = storage_with_index
            .0
            .borrow_mut()
            .value_size(storage_with_index.1)?;
        let size = storage_with_index
            .0
            .borrow_mut()
            .value_at::<u64>(storage_with_index.1, 0)?;

        Ok(Self {
            storage: storage_with_index.0,
            storage_index: storage_with_index.1,
            size,
            capacity: Self::capacity_from_bytes(byte_size),
            phantom_data: std::marker::PhantomData::<T>,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn to_vec() {
        let test_file = TestFile::from("./storage_vec-into_vec.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
    }

    #[test]
    fn capacity() {
        let test_file = TestFile::from("./storage_vec-len.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.capacity(), 64);
    }

    #[test]
    fn iteration() {
        let test_file = TestFile::from("./storage_vec-iteration.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        let mut values: Vec<i64> = vec![];

        for value in vec.to_vec().unwrap() {
            values.push(value);
        }

        assert_eq!(values, vec![1, 3, 5]);
    }

    #[test]
    fn len() {
        let test_file = TestFile::from("./storage_vec-len.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.len(), 0);

        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn push() {
        let test_file = TestFile::from("./storage_vec-push.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(vec.storage_index()),
            Ok(vec![1_i64, 3_i64, 5_i64])
        );
    }

    #[test]
    fn remove() {
        let test_file = TestFile::from("./storage_vec-remove.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(1).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1, 5]));
    }

    #[test]
    fn remove_at_end() {
        let test_file = TestFile::from("./storage_vec-remove_at_end.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(2).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1, 3]));
    }

    #[test]
    fn remove_index_out_of_bounds() {
        let test_file = TestFile::from("./storage_vec-remove_index_out_of_bounds.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.remove(0), Err(DbError::from("index out of bounds")));
    }

    #[test]
    fn remove_size_updated() {
        let test_file = TestFile::from("./storage_vec-remove_size_updated.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(1).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(vec.storage_index()),
            Ok(vec![1_i64, 5_i64])
        );
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::from("./storage_vec-reserve_larger.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        assert_eq!(vec.capacity(), 0);

        vec.reserve(20).unwrap();

        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::from("./storage_vec-reserve_smaller.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.reserve(20).unwrap();
        vec.reserve(10).unwrap();

        assert_eq!(vec.capacity(), 20);
    }

    #[test]
    fn resize_larger() {
        let test_file = TestFile::from("./storage_vec-resize_larger.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(6).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(vec.storage_index()),
            Ok(vec![1_i64, 3_i64, 5_i64, 0, 0, 0])
        );
    }

    #[test]
    fn resize_over_capacity() {
        let test_file = TestFile::from("./storage_vec-resize_over_capacity.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(100).unwrap();

        let mut expected = vec![0_i64; 100];
        expected[0] = 1;
        expected[1] = 3;
        expected[2] = 5;

        assert_eq!(vec.len(), 100);
        assert_eq!(vec.capacity(), 100);

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(vec.storage_index()),
            Ok(expected)
        );
    }

    #[test]
    fn resize_same() {
        let test_file = TestFile::from("./storage_vec-resize_same.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(3).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(vec.storage_index()),
            Ok(vec![1_i64, 3_i64, 5_i64])
        );
    }

    #[test]
    fn resize_smaller() {
        let test_file = TestFile::from("./storage_vec-resize_smaller.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.resize(1).unwrap();

        assert_eq!(
            storage
                .borrow_mut()
                .value::<Vec::<i64>>(vec.storage_index()),
            Ok(vec![1_i64])
        );
    }

    #[test]
    fn set_value() {
        let test_file = TestFile::from("./storage_vec-set_value.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
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
        let test_file = TestFile::from("./storage_vec-set_value_out_of_bounds.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(
            vec.set_value(0, &10),
            Err(DbError::from("index out of bounds"))
        );
    }

    #[test]
    fn shrink_to_fit() {
        let test_file = TestFile::from("./storage_vec-shrink_to_fit.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
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
        let test_file = TestFile::from("./storage_vec-shrink_to_fit_empty.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn try_from_storage_index() {
        let test_file = TestFile::from("./storage_vec-try_from_storage_index.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let index;

        {
            let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
            vec.push(&1).unwrap();
            vec.push(&3).unwrap();
            vec.push(&5).unwrap();
            index = vec.storage_index();
        }

        let mut vec = StorageVec::<i64>::try_from((storage, index)).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
    }

    #[test]
    fn try_from_storage_missing_index() {
        let test_file = TestFile::from("./storage_vec-try_from_storage_missing_index.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        assert_eq!(
            StorageVec::<i64>::try_from((storage, 1)).err().unwrap(),
            DbError::from("index '1' not found")
        );
    }

    #[test]
    fn value() {
        let test_file = TestFile::from("./storage_vec-value.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.value(0), Ok(1));
        assert_eq!(vec.value(1), Ok(3));
        assert_eq!(vec.value(2), Ok(5));
    }

    #[test]
    fn value_out_of_bounds() {
        let test_file = TestFile::from("./storage_vec-value_out_of_bounds.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.value(0), Err(DbError::from("index out of bounds")));
    }
}
