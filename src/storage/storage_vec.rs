use super::{file_storage::FileStorage, serialize::Serialize, Storage};

#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct StorageVec<T: Serialize, S: Storage = FileStorage> {
    storage: std::rc::Rc<S>,
    phantom_data: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T: Serialize, S: Storage> StorageVec<T, S> {
    fn size_t() -> u64 {
        std::mem::size_of::<T>() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn create_empty() {
        let test_file = TestFile::from("./storage_vec-create_empty.agdb");
        let storage =
            std::rc::Rc::new(FileStorage::try_from(test_file.file_name().clone()).unwrap());

        let _vec = StorageVec::<i64> {
            storage,
            phantom_data: std::marker::PhantomData::<i64> {},
        };
    }
}
