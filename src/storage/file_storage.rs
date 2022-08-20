#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct FileStorage {
    filename: String,
}

impl From<&str> for FileStorage {
    fn from(filename: &str) -> Self {
        FileStorage::from(filename.to_string())
    }
}

impl From<String> for FileStorage {
    fn from(filename: String) -> Self {
        FileStorage { filename }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn file_storage() {
        let _storage = FileStorage::default();
    }

    #[test]
    fn create_new_file() {
        let test_file = TestFile::from("./file_storage_test.agdb");
        let _storage = FileStorage::from(test_file.filename);
    }
}
