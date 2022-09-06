use super::file_storage::FileStorage;
use super::serialize::Serialize;
use super::Storage;
use crate::db_error::DbError;

#[allow(dead_code)]
pub(crate) struct StorageVec<T: Serialize, S: Storage = FileStorage> {
    storage: std::rc::Rc<std::cell::RefCell<S>>,
    index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Serialize, S: Storage> StorageVec<T, S> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }

    pub(crate) fn push(&mut self, value: T) -> Result<(), DbError> {
        if self.size == self.capacity {
            self.reallocate(self.capacity * 2)
        }

        self.size += 1;

        Ok(())
    }

    fn reallocate(&mut self, mut new_capacity: u64) {
        if new_capacity < 64 {
            new_capacity = 64;
        }
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
            index,
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
    fn push() {
        let test_file = TestFile::from("./storage_vec-from_file_storage.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage.clone()).unwrap();
        vec.push(1).unwrap();
        vec.push(3).unwrap();
        vec.push(5).unwrap();

        assert_eq!(
            storage.borrow_mut().value::<Vec::<i64>>(vec.index()),
            Ok(vec![1, 3, 5])
        );
    }
}
