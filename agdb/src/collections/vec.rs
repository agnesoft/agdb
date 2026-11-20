use crate::DbError;
use crate::DbErrorType;
use crate::StorageData;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::storage::StorageSlice;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::serialize::u64_at;
use std::marker::PhantomData;

pub trait VecData<T, D, E>
where
    D: StorageData,
{
    fn capacity(&self) -> u64;
    fn len(&self) -> u64;
    fn reallocate(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), E>;
    fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), E>;
    fn remove(&mut self, storage: &mut Storage<D>, index: u64) -> Result<T, E>;
    fn replace(&mut self, storage: &mut Storage<D>, index: u64, value: &T) -> Result<T, E>;
    fn resize(&mut self, storage: &mut Storage<D>, new_len: u64, value: &T) -> Result<(), E>;
    fn value(&self, storage: &Storage<D>, index: u64) -> Result<T, E>;
}

pub trait VecValue<D: StorageData>: Sized {
    fn store(&self, storage: &mut Storage<D>) -> Result<Vec<u8>, DbError>;
    fn load(storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError>;
    fn remove(storage: &mut Storage<D>, bytes: &[u8]) -> Result<(), DbError>;
    fn storage_len() -> u64;
}

impl<D: StorageData> VecValue<D> for u64 {
    fn store(&self, _storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load(_storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove(_storage: &mut Storage<D>, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

impl<D: StorageData> VecValue<D> for i64 {
    fn store(&self, _storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load(_storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove(_storage: &mut Storage<D>, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

impl<D: StorageData> VecValue<D> for String {
    fn store(&self, storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        let index = storage.insert(self)?;
        Ok(index.serialize())
    }

    fn load(storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        let index = StorageIndex::deserialize(bytes)?;
        storage.value(index)
    }

    fn remove(storage: &mut Storage<D>, bytes: &[u8]) -> Result<(), DbError> {
        let index = StorageIndex::deserialize(bytes)?;
        storage.remove(index)
    }

    fn storage_len() -> u64 {
        StorageIndex::serialized_size_static()
    }
}

pub struct DbVecData<T, D, E>
where
    T: VecValue<D>,
    D: StorageData,
    E: From<DbError>,
{
    capacity: u64,
    len: u64,
    storage_index: StorageIndex,
    phantom_data: PhantomData<(T, D, E)>,
}

impl<T, D, E> DbVecData<T, D, E>
where
    T: VecValue<D>,
    D: StorageData,
    E: From<DbError>,
{
    fn offset(index: u64) -> u64 {
        u64::serialized_size_static() + T::storage_len() * index
    }
}

impl<T, D, E> VecData<T, D, E> for DbVecData<T, D, E>
where
    T: VecValue<D>,
    D: StorageData,
    E: From<DbError>,
{
    fn capacity(&self) -> u64 {
        self.capacity
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn reallocate(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), E> {
        storage.resize_value(
            self.storage_index,
            self.len().serialized_size() + T::storage_len() * capacity,
        )?;

        self.capacity = capacity;

        Ok(())
    }

    fn remove(&mut self, storage: &mut Storage<D>, index: u64) -> Result<T, E> {
        let offset_from = Self::offset(index + 1);
        let offset_to = Self::offset(index);
        let move_len = T::storage_len() * (self.len() - index - 1);
        let bytes = storage
            .value_as_bytes_at_size(self.storage_index, Self::offset(index), T::storage_len())?
            .to_vec();

        let value = T::load(storage, &bytes)?;
        let len = self.len - 1;

        let id = storage.transaction();
        T::remove(storage, &bytes)?;
        storage.move_at(self.storage_index, offset_from, offset_to, move_len)?;
        storage.insert_at(self.storage_index, 0, &len)?;
        storage.commit(id)?;

        self.len = len;

        Ok(value)
    }

    fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), E> {
        let id = storage.transaction();

        for index in 0..self.len() {
            let bytes = storage
                .value_as_bytes_at_size(self.storage_index, Self::offset(index), T::storage_len())?
                .to_vec();
            T::remove(storage, &bytes)?;
        }

        storage.remove(self.storage_index)?;
        storage.commit(id)?;
        Ok(())
    }

    fn replace(&mut self, storage: &mut Storage<D>, index: u64, value: &T) -> Result<T, E> {
        let old_bytes = storage
            .value_as_bytes_at_size(self.storage_index, Self::offset(index), T::storage_len())?
            .to_vec();
        let old_value = T::load(storage, &old_bytes)?;

        let id = storage.transaction();
        T::remove(storage, &old_bytes)?;
        let bytes = value.store(storage)?;
        storage.insert_bytes_at(self.storage_index, Self::offset(index), &bytes)?;
        storage.commit(id)?;

        Ok(old_value)
    }

    fn resize(&mut self, storage: &mut Storage<D>, new_len: u64, value: &T) -> Result<(), E> {
        let id = storage.transaction();

        if self.len() < new_len {
            let mut bytes =
                Vec::with_capacity(((new_len - self.len()) * T::storage_len()) as usize);

            for _ in self.len()..new_len {
                bytes.extend(value.store(storage)?);
            }

            storage.insert_bytes_at(self.storage_index, Self::offset(self.len()), &bytes)?;
        }

        for index in new_len..self.len() {
            let old_bytes = storage
                .value_as_bytes_at_size(self.storage_index, Self::offset(index), T::storage_len())?
                .to_vec();
            T::remove(storage, &old_bytes)?;
        }

        storage.insert_at(self.storage_index, 0, &new_len)?;
        storage.commit(id)?;

        self.len = new_len;

        Ok(())
    }

    fn value(&self, storage: &Storage<D>, index: u64) -> Result<T, E> {
        let bytes = storage.value_as_bytes_at_size(
            self.storage_index,
            Self::offset(index),
            T::storage_len(),
        )?;

        Ok(T::load(storage, &bytes)?)
    }
}

pub struct VecImpl<T, D, Data, E>
where
    T: VecValue<D>,
    D: StorageData,
    Data: VecData<T, D, E>,
    E: From<DbError>,
{
    phantom_data: PhantomData<(T, D, E)>,
    data: Data,
}

pub struct VecIterator<'a, T, D, Data, E>
where
    T: VecValue<D>,
    D: StorageData,
    Data: VecData<T, D, E>,
    E: From<DbError>,
{
    pub index: u64,
    pub vec: &'a VecImpl<T, D, Data, E>,
    pub storage: &'a Storage<D>,
}

pub type DbVec<T, D> = VecImpl<T, D, DbVecData<T, D, DbError>, DbError>;

impl<T, D, Data, E> Iterator for VecIterator<'_, T, D, Data, E>
where
    T: VecValue<D>,
    D: StorageData,
    Data: VecData<T, D, E>,
    E: From<DbError>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.vec.value(self.storage, self.index).ok();
        self.index += 1;

        value
    }
}

impl<T, D, Data, E> VecImpl<T, D, Data, E>
where
    T: VecValue<D>,
    D: StorageData,
    Data: VecData<T, D, E>,
    E: From<DbError>,
{
    pub fn capacity(&self) -> u64 {
        self.data.capacity()
    }

    pub fn len(&self) -> u64 {
        self.data.len()
    }

    pub fn iter<'a>(&'a self, storage: &'a Storage<D>) -> VecIterator<'a, T, D, Data, E> {
        VecIterator {
            index: 0,
            vec: self,
            storage,
        }
    }

    pub fn push(&mut self, storage: &mut Storage<D>, value: &T) -> Result<(), E> {
        if self.data.len() == self.data.capacity() {
            self.data.reallocate(
                storage,
                match self.capacity() {
                    0 => 1,
                    1 => 2,
                    _ => self.capacity() + self.capacity() / 2,
                },
            )?;
        }

        self.data.resize(storage, self.data.len() + 1, value)
    }

    pub fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), E> {
        self.data.remove_from_storage(storage)
    }

    pub fn remove(&mut self, storage: &mut Storage<D>, index: u64) -> Result<T, E> {
        self.validate_index(index)?;
        self.data.remove(storage, index)
    }

    pub fn replace(&mut self, storage: &mut Storage<D>, index: u64, value: &T) -> Result<T, E> {
        self.validate_index(index)?;
        self.data.replace(storage, index, value)
    }

    pub fn reserve(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), E> {
        if self.capacity() < capacity {
            self.data.reallocate(storage, capacity)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, storage: &mut Storage<D>, new_len: u64, value: &T) -> Result<(), E> {
        self.reserve(storage, new_len)?;
        self.data.resize(storage, new_len, value)
    }

    pub fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), E> {
        self.data.reallocate(storage, self.len())
    }

    pub fn value(&self, storage: &Storage<D>, index: u64) -> Result<T, E> {
        self.validate_index(index)?;
        self.data.value(storage, index)
    }

    fn validate_index(&self, index: u64) -> Result<(), E> {
        if self.len() <= index {
            return Err(E::from(DbError::collections(
                DbErrorType::OutOfBounds,
                format!("Index ({index}) out of bounds ({})", self.len()),
            )));
        }

        Ok(())
    }
}

impl<T, D> DbVec<T, D>
where
    T: VecValue<D>,
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        let storage_index = storage.insert(&0_u64)?;

        Ok(Self {
            phantom_data: PhantomData,
            data: DbVecData {
                capacity: 0,
                len: 0,
                storage_index,
                phantom_data: PhantomData,
            },
        })
    }

    pub fn from_storage(
        storage: &Storage<D>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let data_len = storage.value_size(storage_index)?;
        let len = u64_at(&storage.value_as_bytes_at_size(storage_index, 0, 8)?, 0);
        let capacity = data_len.saturating_sub(u64::serialized_size_static()) / T::storage_len();

        Ok(DbVec {
            phantom_data: PhantomData,
            data: DbVecData {
                capacity,
                len,
                storage_index,
                phantom_data: PhantomData,
            },
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index
    }

    pub fn bytes_at<'a>(
        &self,
        storage: &'a Storage<D>,
        index: u64,
        count: u64,
    ) -> Result<StorageSlice<'a>, DbError> {
        storage.value_as_bytes_at_size(
            self.data.storage_index,
            DbVecData::<T, D, DbError>::offset(index),
            count * T::storage_len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::assert_io_failures;
    use crate::collections::tests::storage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    type TestVec<T> = DbVec<T, FileStorageImpl<TestPlatform>>;

    fn strings(storage: &mut TestStorage, values: &[&str]) -> TestVec<String> {
        let mut vec = TestVec::<String>::new(storage).unwrap();

        for value in values {
            vec.push(storage, &value.to_string()).unwrap();
        }

        vec
    }

    fn values(vec: &TestVec<String>, storage: &TestStorage) -> Vec<String> {
        vec.iter(storage).collect()
    }

    fn out_of_bounds(index: u64, len: u64) -> DbError {
        DbError::collections(
            DbErrorType::OutOfBounds,
            format!("Index ({index}) out of bounds ({len})"),
        )
    }

    mod vec_value {
        use super::*;

        mod store {
            use super::*;

            #[test]
            fn store_should_serialize_numbers() {
                let mut storage = storage();

                assert_eq!(1_u64.store(&mut storage), Ok(1_u64.serialize()));
                assert_eq!((-1_i64).store(&mut storage), Ok((-1_i64).serialize()));
            }

            #[test]
            fn store_string_should_insert_it_into_storage() {
                let mut storage = storage();

                let bytes = "Hello".to_string().store(&mut storage).unwrap();

                let index = StorageIndex::deserialize(&bytes).unwrap();
                assert_eq!(storage.value::<String>(index), Ok("Hello".to_string()));
            }
        }

        mod load {
            use super::*;

            #[test]
            fn load_should_read_stored_value() {
                let mut storage = storage();
                let u64_bytes = 1_u64.store(&mut storage).unwrap();
                let i64_bytes = (-1_i64).store(&mut storage).unwrap();
                let string_bytes = "Hello".to_string().store(&mut storage).unwrap();

                assert_eq!(u64::load(&storage, &u64_bytes), Ok(1));
                assert_eq!(i64::load(&storage, &i64_bytes), Ok(-1));
                assert_eq!(
                    String::load(&storage, &string_bytes),
                    Ok("Hello".to_string())
                );
            }

            #[test]
            fn load_short_bytes_should_fail() {
                let storage = storage();

                assert!(<u64 as VecValue<_>>::load(&storage, &[1]).is_err());
                assert!(<i64 as VecValue<_>>::load(&storage, &[1]).is_err());
                assert!(<String as VecValue<_>>::load(&storage, &[1]).is_err());
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_string_should_remove_it_from_storage() {
                let mut storage = storage();
                let bytes = "Hello".to_string().store(&mut storage).unwrap();

                <String as VecValue<_>>::remove(&mut storage, &bytes).unwrap();

                let index = StorageIndex::deserialize(&bytes).unwrap();
                assert!(storage.value::<String>(index).is_err());
            }

            #[test]
            fn remove_number_should_do_nothing() {
                let mut storage = storage();
                let len = storage.len();

                <u64 as VecValue<_>>::remove(&mut storage, &1_u64.serialize()).unwrap();
                <i64 as VecValue<_>>::remove(&mut storage, &1_i64.serialize()).unwrap();

                assert_eq!(storage.len(), len);
            }

            #[test]
            fn remove_string_short_bytes_should_fail() {
                let mut storage = storage();

                assert!(<String as VecValue<_>>::remove(&mut storage, &[1]).is_err());
            }
        }

        mod storage_len {
            use super::*;

            #[test]
            fn storage_len_should_be_8_bytes() {
                type D = FileStorageImpl<TestPlatform>;

                assert_eq!(<u64 as VecValue<D>>::storage_len(), 8);
                assert_eq!(<i64 as VecValue<D>>::storage_len(), 8);
                assert_eq!(<String as VecValue<D>>::storage_len(), 8);
            }
        }
    }

    mod vec_iterator {
        use super::*;

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_yield_values_in_order_then_none() {
                    let mut storage = storage();
                    let vec = strings(&mut storage, &["a", "b"]);
                    let mut iter = vec.iter(&storage);

                    assert_eq!(iter.next(), Some("a".to_string()));
                    assert_eq!(iter.next(), Some("b".to_string()));
                    assert_eq!(iter.next(), None);
                }
            }
        }
    }

    mod vec_impl {
        use super::*;

        mod capacity {
            use super::*;

            #[test]
            fn capacity_of_new_vec_should_be_zero() {
                let mut storage = storage();

                assert_eq!(strings(&mut storage, &[]).capacity(), 0);
            }
        }

        mod len {
            use super::*;

            #[test]
            fn len_should_count_values() {
                let mut storage = storage();

                assert_eq!(strings(&mut storage, &[]).len(), 0);
                assert_eq!(strings(&mut storage, &["a", "b", "c"]).len(), 3);
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_yield_all_values_in_order() {
                let mut storage = storage();
                let vec = strings(&mut storage, &["Hello", ", ", "World", "!"]);

                assert_eq!(values(&vec, &storage), vec!["Hello", ", ", "World", "!"]);
            }
        }

        mod push {
            use super::*;

            #[test]
            fn push_should_append_value() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a"]);

                vec.push(&mut storage, &"b".to_string()).unwrap();

                assert_eq!(values(&vec, &storage), vec!["a", "b"]);
            }

            #[test]
            fn push_should_grow_capacity_by_half_from_two() {
                let mut storage = storage();
                let mut vec = TestVec::<u64>::new(&mut storage).unwrap();
                let mut capacities = vec![];

                for value in 0..7 {
                    vec.push(&mut storage, &value).unwrap();
                    capacities.push(vec.capacity());
                }

                assert_eq!(capacities, vec![1, 2, 3, 4, 6, 6, 9]);
            }

            #[test]
            fn push_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a"]),
                    |storage, vec| vec.push(storage, &"b".to_string()),
                );
            }
        }

        mod remove_from_storage {
            use super::*;

            #[test]
            fn remove_from_storage_should_remove_vec_and_its_values() {
                let mut storage = storage();
                let len = storage.len();
                let vec = strings(&mut storage, &["a long value not fitting in an index"]);

                vec.remove_from_storage(&mut storage).unwrap();

                storage.optimize_storage().unwrap();
                assert_eq!(storage.len(), len);
            }

            #[test]
            fn remove_from_storage_failure_should_fail() {
                assert_io_failures(
                    |storage| Some(strings(storage, &["a", "b"])),
                    |storage, vec| vec.take().unwrap().remove_from_storage(storage),
                );
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_should_return_value_and_shift_following_ones() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b", "c", "d"]);

                assert_eq!(vec.remove(&mut storage, 1), Ok("b".to_string()));

                assert_eq!(values(&vec, &storage), vec!["a", "c", "d"]);
            }

            #[test]
            fn remove_last_should_keep_other_values() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b", "c"]);

                assert_eq!(vec.remove(&mut storage, 2), Ok("c".to_string()));

                assert_eq!(values(&vec, &storage), vec!["a", "b"]);
            }

            #[test]
            fn remove_last_at_capacity_should_keep_other_values() {
                let mut storage = storage();
                let mut vec = TestVec::<u64>::new(&mut storage).unwrap();
                for value in 1..4 {
                    vec.push(&mut storage, &value).unwrap();
                }
                vec.shrink_to_fit(&mut storage).unwrap();

                assert_eq!(vec.remove(&mut storage, 2), Ok(3));

                assert_eq!(vec.iter(&storage).collect::<Vec<u64>>(), vec![1, 2]);
            }

            #[test]
            fn remove_rolled_back_should_keep_committed_values() {
                // GIVEN a committed vec followed by another value
                let mut storage = storage();
                let mut vec = TestVec::<i64>::new(&mut storage).unwrap();
                vec.push(&mut storage, &1).unwrap();
                vec.push(&mut storage, &2).unwrap();
                let other = storage.insert(&10_i64).unwrap();

                // WHEN the last value is removed in a transaction that is rolled back
                let _id = storage.transaction();
                vec.remove(&mut storage, 1).unwrap();
                storage.recover().unwrap();

                // THEN the reloaded vec and the other value are as committed
                let vec = TestVec::<i64>::from_storage(&storage, vec.storage_index()).unwrap();
                assert_eq!(vec.iter(&storage).collect::<Vec<i64>>(), vec![1, 2]);
                assert_eq!(storage.value::<i64>(other), Ok(10));
            }

            #[test]
            fn remove_out_of_bounds_should_fail() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a"]);

                assert_eq!(vec.remove(&mut storage, 1), Err(out_of_bounds(1, 1)));
            }

            #[test]
            fn remove_failure_should_fail_and_keep_len() {
                assert_io_failures(
                    |storage| strings(storage, &["a", "b"]),
                    |storage, vec| {
                        let result = vec.remove(storage, 0);
                        assert_eq!(vec.len(), if result.is_ok() { 1 } else { 2 });
                        result
                    },
                );
            }
        }

        mod replace {
            use super::*;

            #[test]
            fn replace_should_return_old_value_and_store_new_one() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b", "c"]);

                assert_eq!(
                    vec.replace(&mut storage, 1, &"x".to_string()),
                    Ok("b".to_string())
                );

                assert_eq!(values(&vec, &storage), vec!["a", "x", "c"]);
            }

            #[test]
            fn replace_out_of_bounds_should_fail() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &[]);

                assert_eq!(
                    vec.replace(&mut storage, 0, &"x".to_string()),
                    Err(out_of_bounds(0, 0))
                );
            }

            #[test]
            fn replace_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a"]),
                    |storage, vec| vec.replace(storage, 0, &"x".to_string()),
                );
            }
        }

        mod reserve {
            use super::*;

            #[test]
            fn reserve_larger_should_grow_capacity() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a"]);

                vec.reserve(&mut storage, 20).unwrap();

                assert_eq!(vec.capacity(), 20);
                assert_eq!(values(&vec, &storage), vec!["a"]);
            }

            #[test]
            fn reserve_smaller_should_keep_capacity() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &[]);
                vec.reserve(&mut storage, 20).unwrap();

                vec.reserve(&mut storage, 10).unwrap();

                assert_eq!(vec.capacity(), 20);
            }

            #[test]
            fn reserve_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a"]),
                    |storage, vec| vec.reserve(storage, 20),
                );
            }
        }

        mod resize {
            use super::*;

            #[test]
            fn resize_larger_should_append_copies_of_value() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b", "c", "d"]);

                vec.resize(&mut storage, 6, &" ".to_string()).unwrap();

                assert_eq!(values(&vec, &storage), vec!["a", "b", "c", "d", " ", " "]);
            }

            #[test]
            fn resize_larger_should_store_each_copy() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &[]);
                vec.resize(&mut storage, 2, &"a".to_string()).unwrap();

                vec.replace(&mut storage, 0, &"b".to_string()).unwrap();

                assert_eq!(values(&vec, &storage), vec!["b", "a"]);
            }

            #[test]
            fn resize_over_capacity_should_grow_capacity_to_len() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b", "c", "d"]);

                vec.resize(&mut storage, 100, &" ".to_string()).unwrap();

                let mut expected = vec!["a", "b", "c", "d"];
                expected.resize(100, " ");
                assert_eq!(vec.len(), 100);
                assert_eq!(vec.capacity(), 100);
                assert_eq!(values(&vec, &storage), expected);
            }

            #[test]
            fn resize_same_should_keep_values() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b"]);

                vec.resize(&mut storage, 2, &String::new()).unwrap();

                assert_eq!(values(&vec, &storage), vec!["a", "b"]);
            }

            #[test]
            fn resize_smaller_should_remove_values_past_len() {
                let mut storage = storage();
                let len = storage.len();
                let mut vec = strings(&mut storage, &[]);
                vec.reserve(&mut storage, 4).unwrap();
                let empty_vec_len = storage.len() - len;
                vec.push(&mut storage, &"a".to_string()).unwrap();
                vec.push(
                    &mut storage,
                    &"a long value not fitting in an index".to_string(),
                )
                .unwrap();

                vec.resize(&mut storage, 0, &String::new()).unwrap();

                storage.optimize_storage().unwrap();
                assert_eq!(vec.len(), 0);
                assert_eq!(storage.len(), len + empty_vec_len);
            }

            #[test]
            fn resize_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a", "b"]),
                    |storage, vec| vec.resize(storage, 1, &String::new()),
                );
                assert_io_failures(
                    |storage| strings(storage, &["a", "b"]),
                    |storage, vec| vec.resize(storage, 10, &String::new()),
                );
            }
        }

        mod shrink_to_fit {
            use super::*;

            #[test]
            fn shrink_to_fit_should_reduce_capacity_to_len() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &["a", "b", "c", "d", "e"]);
                assert_eq!(vec.capacity(), 6);

                vec.shrink_to_fit(&mut storage).unwrap();

                assert_eq!(vec.capacity(), 5);
                assert_eq!(values(&vec, &storage), vec!["a", "b", "c", "d", "e"]);
            }

            #[test]
            fn shrink_to_fit_empty_should_keep_zero_capacity() {
                let mut storage = storage();
                let mut vec = strings(&mut storage, &[]);

                vec.shrink_to_fit(&mut storage).unwrap();

                assert_eq!(vec.capacity(), 0);
            }

            #[test]
            fn shrink_to_fit_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a", "b", "c", "d", "e"]),
                    |storage, vec| vec.shrink_to_fit(storage),
                );
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_should_return_value_at_index() {
                let mut storage = storage();
                let vec = strings(&mut storage, &["a", "b"]);

                assert_eq!(vec.value(&storage, 0), Ok("a".to_string()));
                assert_eq!(vec.value(&storage, 1), Ok("b".to_string()));
            }

            #[test]
            fn value_out_of_bounds_should_fail() {
                let mut storage = storage();
                let vec = strings(&mut storage, &["a"]);

                assert_eq!(vec.value(&storage, 1), Err(out_of_bounds(1, 1)));
            }

            #[test]
            fn value_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a"]),
                    |storage, vec| vec.value(storage, 0),
                );
            }
        }
    }

    mod db_vec {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_create_empty_vec_in_storage() {
                let mut storage = storage();

                let vec = TestVec::<u64>::new(&mut storage).unwrap();

                assert_eq!(vec.len(), 0);
                assert_eq!(storage.value::<u64>(vec.storage_index()), Ok(0));
            }

            #[test]
            fn new_failure_should_fail() {
                assert_io_failures(|_| (), |storage, _| TestVec::<u64>::new(storage));
            }
        }

        mod from_storage {
            use super::*;

            #[test]
            fn from_storage_should_load_values_and_capacity() {
                let mut storage = storage();
                let vec = strings(&mut storage, &["Hello", ", ", "World", "!", "?"]);

                let vec = TestVec::<String>::from_storage(&storage, vec.storage_index()).unwrap();

                assert_eq!(vec.capacity(), 6);
                assert_eq!(
                    values(&vec, &storage),
                    vec!["Hello", ", ", "World", "!", "?"]
                );
            }

            #[test]
            fn from_storage_of_signed_values_should_load_capacity() {
                let mut storage = storage();
                let mut vec = TestVec::<i64>::new(&mut storage).unwrap();
                for value in [10, 20, 30, 40] {
                    vec.push(&mut storage, &value).unwrap();
                }

                let vec = TestVec::<i64>::from_storage(&storage, vec.storage_index()).unwrap();

                assert_eq!(vec.len(), 4);
                assert_eq!(vec.capacity(), 4);
                assert_eq!(
                    vec.iter(&storage).collect::<Vec<i64>>(),
                    vec![10, 20, 30, 40]
                );
            }

            #[test]
            fn from_storage_missing_index_should_fail() {
                let storage = storage();

                assert_eq!(
                    TestVec::<String>::from_storage(&storage, StorageIndex::from(1_u64))
                        .err()
                        .unwrap(),
                    DbError::collections(DbErrorType::NotFound, "Index (1) not found")
                );
            }

            #[test]
            fn from_storage_short_record_should_fail() {
                let mut storage = storage();
                let index = storage.insert_bytes(&[1]).unwrap();

                assert!(TestVec::<String>::from_storage(&storage, index).is_err());
            }

            #[test]
            fn from_storage_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a"]).storage_index(),
                    |storage, index| TestVec::<String>::from_storage(storage, *index),
                );
            }
        }

        mod bytes_at {
            use super::*;

            #[test]
            fn bytes_at_should_return_stored_values_from_index() {
                let mut storage = storage();
                let mut vec = TestVec::<u64>::new(&mut storage).unwrap();
                vec.push(&mut storage, &1).unwrap();
                vec.push(&mut storage, &2).unwrap();
                vec.push(&mut storage, &3).unwrap();

                assert_eq!(
                    vec.bytes_at(&storage, 1, 2).unwrap().to_vec(),
                    [2_u64.serialize(), 3_u64.serialize()].concat()
                );
            }

            #[test]
            fn bytes_at_failure_should_fail() {
                assert_io_failures(
                    |storage| strings(storage, &["a", "b"]),
                    |storage, vec| vec.bytes_at(storage, 1, 1).map(|bytes| bytes.to_vec()),
                );
            }
        }
    }
}
