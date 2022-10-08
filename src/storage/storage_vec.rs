use super::vec_iterator::VecIterator;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::FileStorage;
use agdb_storage::Storage;

pub(crate) struct StorageVec<T, Data = FileStorage>
where
    T: Serialize,
    Data: Storage,
{
    storage: std::rc::Rc<std::cell::RefCell<Data>>,
    storage_index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T, Data> StorageVec<T, Data>
where
    T: Serialize,
    Data: Storage,
{
    pub(crate) fn capacity(&self) -> u64 {
        self.capacity
    }

    pub(crate) fn len(&self) -> u64 {
        self.size
    }

    pub(crate) fn iter(&self) -> VecIterator<T, Data> {
        VecIterator::<T, Data> {
            index: 0,
            vec: self,
            phantom_data: std::marker::PhantomData,
        }
    }

    pub(crate) fn push(&mut self, value: &T) -> Result<(), DbError> {
        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.transaction();

        if self.size == self.capacity {
            let current_capacity = self.capacity;
            Self::reallocate(
                &mut self.capacity,
                std::cmp::max(current_capacity * 2, 64),
                &mut ref_storage,
                self.storage_index,
            )?;
        }

        ref_storage.insert_at(self.storage_index, Self::value_offset(self.size), value)?;
        self.size += 1;
        ref_storage.insert_at(self.storage_index, 0, &self.size)?;
        ref_storage.commit()
    }

    pub(crate) fn remove(&mut self, index: u64) -> Result<(), DbError> {
        if self.size <= index {
            return Err(DbError::from("index out of bounds"));
        }

        let offset_from = Self::value_offset(index + 1);
        let offset_to = Self::value_offset(index);
        let size = Self::value_offset(self.size) - offset_from;

        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.transaction();
        ref_storage.move_at(self.storage_index, offset_from, offset_to, size)?;
        self.size -= 1;
        ref_storage.insert_at(self.storage_index, 0, &self.size)?;
        ref_storage.commit()
    }

    pub(crate) fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if capacity <= self.capacity {
            return Ok(());
        }

        let mut ref_storage = self.storage.borrow_mut();
        Self::reallocate(
            &mut self.capacity,
            capacity,
            &mut ref_storage,
            self.storage_index,
        )
    }

    pub(crate) fn resize(&mut self, size: u64) -> Result<(), DbError> {
        if self.size == size {
            return Ok(());
        }

        let mut ref_storage = self.storage.borrow_mut();
        ref_storage.transaction();

        if size < self.size {
            let offset = Self::value_offset(size);
            let byte_size = Self::value_offset(self.size) - offset;
            ref_storage.insert_at(self.storage_index, offset, &vec![0_u8; byte_size as usize])?;
        } else if self.capacity < size {
            Self::reallocate(
                &mut self.capacity,
                size,
                &mut ref_storage,
                self.storage_index,
            )?;
        }

        self.size = size;
        ref_storage.insert_at(self.storage_index, 0, &self.size)?;
        ref_storage.commit()
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
        let mut ref_storage = self.storage.borrow_mut();
        Self::reallocate(
            &mut self.capacity,
            self.size,
            &mut ref_storage,
            self.storage_index,
        )
    }

    pub(crate) fn storage_index(&self) -> i64 {
        self.storage_index
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_vec(&self) -> Result<Vec<T>, DbError> {
        self.storage.borrow_mut().value(self.storage_index)
    }

    pub(crate) fn value(&self, index: u64) -> Result<T, DbError> {
        if self.size <= index {
            return Err(DbError::from("index out of bounds"));
        }

        self.storage
            .borrow_mut()
            .value_at::<T>(self.storage_index, Self::value_offset(index))
    }

    pub(crate) fn value_offset(index: u64) -> u64 {
        u64::serialized_size() + index * T::serialized_size()
    }

    fn reallocate(
        capacity: &mut u64,
        new_capacity: u64,
        storage: &mut std::cell::RefMut<Data>,
        index: i64,
    ) -> Result<(), DbError> {
        *capacity = new_capacity;
        storage.resize_value(index, Self::value_offset(new_capacity))
    }

    fn capacity_from_bytes(len: u64) -> u64 {
        (len - u64::serialized_size()) / T::serialized_size()
    }
}

impl<T, Data> TryFrom<std::rc::Rc<std::cell::RefCell<Data>>> for StorageVec<T, Data>
where
    T: Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: std::rc::Rc<std::cell::RefCell<Data>>) -> Result<Self, Self::Error> {
        let index = storage.borrow_mut().insert(&0_u64)?;

        Ok(Self {
            storage,
            storage_index: index,
            size: 0,
            capacity: 0,
            phantom_data: std::marker::PhantomData,
        })
    }
}

impl<T: Serialize, Data: Storage> TryFrom<(std::rc::Rc<std::cell::RefCell<Data>>, i64)>
    for StorageVec<T, Data>
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (std::rc::Rc<std::cell::RefCell<Data>>, i64),
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
            phantom_data: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb_test_file::TestFile;

    #[test]
    fn capacity() {
        let test_file = TestFile::new();
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
    fn iter() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();
        vec.push(&1).unwrap();
        vec.push(&3).unwrap();
        vec.push(&5).unwrap();

        assert_eq!(vec.iter().collect::<Vec<i64>>(), vec![1_i64, 3_i64, 5_i64]);
    }

    #[test]
    fn len() {
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.remove(0), Err(DbError::from("index out of bounds")));
    }

    #[test]
    fn remove_size_updated() {
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.capacity(), 0);

        vec.shrink_to_fit().unwrap();

        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn to_vec() {
        let test_file = TestFile::new();
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
    fn try_from_storage_index() {
        let test_file = TestFile::new();
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

        let vec = StorageVec::<i64>::try_from((storage, index)).unwrap();

        assert_eq!(vec.to_vec(), Ok(vec![1_i64, 3_i64, 5_i64]));
    }

    #[test]
    fn try_from_storage_missing_index() {
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
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
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let vec = StorageVec::<i64>::try_from(storage).unwrap();

        assert_eq!(vec.value(0), Err(DbError::from("index out of bounds")));
    }
}
