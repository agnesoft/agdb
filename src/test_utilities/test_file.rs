use std::path::Path;

pub(crate) struct TestFile {
    filename: String,
}

fn remove_file_if_exists(filename: &String) {
    if Path::new(filename).exists() {
        std::fs::remove_file(filename).unwrap();
    }
}

impl TestFile {
    #[allow(dead_code)]
    pub(crate) fn file_name(&self) -> &String {
        &self.filename
    }
}

impl From<&str> for TestFile {
    fn from(filename: &str) -> Self {
        TestFile::from(filename.to_string())
    }
}

impl From<String> for TestFile {
    fn from(filename: String) -> Self {
        remove_file_if_exists(&filename);

        TestFile { filename }
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        remove_file_if_exists(&self.filename);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn ensure_file(filename: &str) {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filename)
            .unwrap();
    }

    #[test]
    fn created_from_str_ref() {
        let filename = "./test_file_test_file01";
        let _test_file = TestFile::from(filename);
    }

    #[test]
    fn created_from_string() {
        let filename = "./test_file_test_file02".to_string();
        let _test_file = TestFile::from(filename);
    }

    #[test]
    fn get_file_name() {
        let filename = "./test_file_test_file03";
        let test_file = TestFile::from(filename);

        assert_eq!(test_file.file_name(), filename);
    }

    #[test]
    fn existing_file_is_deleted_on_construction() {
        let filename = "./test_file_test_file04";
        ensure_file(filename);
        let _test_file = TestFile::from(filename);
        assert!(!Path::new(filename).exists());
    }

    #[test]
    fn file_is_deleted_on_destruction() {
        let filename = "./test_file_test_file05";

        {
            let _test_file = TestFile::from(filename);
            ensure_file(filename);
        }

        assert!(!Path::new(filename).exists());
    }
}
