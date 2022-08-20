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
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn created_from_str_ref() {
        let filename = "./test_file_test_file";
        let _test_file = TestFile::from(filename);
    }

    #[test]
    fn created_from_string() {
        let filename = "./test_file_test_file".to_string();
        let _test_file = TestFile::from(filename);
    }

    #[test]
    fn existing_file_is_deleted_on_construction() {
        let filename = "./test_file_test_file";
        let file_path = Path::new(filename);
        File::create(filename).unwrap();
        assert!(file_path.exists());

        let _test_file = TestFile::from(filename);
        assert!(!file_path.exists());
    }

    #[test]
    fn file_is_deleted_on_destruction() {
        let filename = "./test_file_test_file";
        let file_path = Path::new(filename);

        {
            let _test_file = TestFile::from(filename);
            File::create(filename).unwrap();
            assert!(file_path.exists());
        }

        assert!(!file_path.exists());
    }

    #[test]
    fn get_file_name() {
        let filename = "./test_file_test_file";
        let test_file = TestFile::from(filename);

        assert_eq!(test_file.file_name(), filename);
    }
}
