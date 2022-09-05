use super::{file_storage::FileStorage, serialize::Serialize, Storage};

#[allow(dead_code)]
pub(crate) struct StorageVec<T: Serialize, S: Storage = FileStorage> {
    storage: std::rc::Rc<S>,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T: Serialize, S: Storage> From<std::rc::Rc<S>> for StorageVec<T, S> {
    fn from(storage: std::rc::Rc<S>) -> Self {
        Self {
            storage,
            phantom_data: std::marker::PhantomData::<T>,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn from_file_storage() {
        let test_file = TestFile::from("./storage_vec-from_file_storage.agdb");
        let storage =
            std::rc::Rc::new(FileStorage::try_from(test_file.file_name().clone()).unwrap());

        let _vec = StorageVec::<i64>::from(storage);
    }
}
