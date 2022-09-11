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
    fn as_vec(&mut self) -> Result<Vec<T>, DbError> {
        self.storage.borrow_mut().value(self.storage_index)
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

    pub(crate) fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        if self.size <= index {
            return Err(DbError::from("index out of bounds"));
        }

        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, Self::value_offset(index), value)
    }

    pub(crate) fn storage_index(&self) -> i64 {
        self.storage_index
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
        std::mem::size_of::<u64>() as u64 + index * std::mem::size_of::<T>() as u64
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn into_vec() {
        let test_file = TestFile::from("./storage_vec-into_vec.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.as_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
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

        for value in vec.as_vec().unwrap() {
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

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(1).unwrap();

        assert_eq!(vec.as_vec(), Ok(vec![1, 5]));
    }

    #[test]
    fn remove_at_end() {
        let test_file = TestFile::from("./storage_vec-remove_at_end.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        vec.remove(2).unwrap();

        assert_eq!(vec.as_vec(), Ok(vec![1, 3]));
    }

    #[test]
    fn remove_index_out_of_bounds() {
        let test_file = TestFile::from("./storage_vec-remove_index_out_of_bounds.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();

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
