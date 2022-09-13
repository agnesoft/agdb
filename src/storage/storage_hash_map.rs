use super::file_storage::FileStorage;
use super::serialize::Serialize;
use super::Storage;
use crate::DbError;

#[allow(dead_code)]
pub(crate) struct HashMap<K: Serialize, T: Serialize, S: Storage = FileStorage> {
    storage: std::rc::Rc<std::cell::RefCell<S>>,
    storage_index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<(K, T)>,
}

impl<K: Serialize, T: Serialize, S: Storage> TryFrom<std::rc::Rc<std::cell::RefCell<S>>>
    for HashMap<K, T, S>
{
    type Error = DbError;

    fn try_from(storage: std::rc::Rc<std::cell::RefCell<S>>) -> Result<Self, Self::Error> {
        let index = storage.borrow_mut().insert(&0_u64)?;

        Ok(Self {
            storage,
            storage_index: index,
            size: 0,
            capacity: 0,
            phantom_data: std::marker::PhantomData::<(K, T)>,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn try_from() {
        let test_file = TestFile::from("./storage_hash_map-try_from.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut _map = HashMap::<i64, i64>::try_from(storage).unwrap();
    }
}
