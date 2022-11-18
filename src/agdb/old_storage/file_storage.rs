use super::Storage;
use crate::utilities::serialize::Serialize;
use crate::DbError;
use crate::DbIndex;

pub struct FileStorage {}

impl FileStorage {
    pub fn new(filename: &String) -> Result<FileStorage, DbError> {
        Ok(FileStorage {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        //let index1 = storage.insert(&String::new()).unwrap();
    }
}
