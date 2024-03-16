use std::panic::Location;
use std::path::Path;

pub struct TestFile {
    filename: String,
}

impl TestFile {
    pub fn file_name(&self) -> &String {
        &self.filename
    }

    #[track_caller]
    pub fn new() -> Self {
        let caller = Location::caller();
        let file = format!(
            "./{}.{}.{}.testfile",
            Path::new(caller.file())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            caller.line(),
            caller.column()
        );

        TestFile::from(file)
    }

    pub fn hidden_filename(filename: &String) -> String {
        let path = Path::new(filename);
        let name: String = path.file_name().unwrap().to_str().unwrap().to_string();
        let parent = path.parent().unwrap();

        parent
            .join(Path::new(&(".".to_string() + &name)))
            .to_str()
            .unwrap()
            .to_string()
    }

    fn remove_file_if_exists(filename: &String) {
        if Path::new(filename).exists() {
            std::fs::remove_file(filename).unwrap();
        }
    }
}

impl Default for TestFile {
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        Self::remove_file_if_exists(&self.filename);
        Self::remove_file_if_exists(&Self::hidden_filename(&self.filename));
    }
}

impl From<&str> for TestFile {
    fn from(filename: &str) -> Self {
        TestFile::from(filename.to_string())
    }
}

impl From<String> for TestFile {
    fn from(filename: String) -> Self {
        Self::remove_file_if_exists(&filename);
        Self::remove_file_if_exists(&Self::hidden_filename(&filename));

        TestFile { filename }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;

    fn ensure_file(filename: &str) {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(filename)
            .unwrap();
    }

    #[test]
    fn default() {
        let caller = Location::caller();
        let current_source_file = Path::new(caller.file())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let test_file = TestFile::default();
        assert!(!test_file.file_name().is_empty());
        assert!(test_file.file_name().contains(current_source_file));
    }

    #[test]
    fn created_from_str_ref() {
        let filename = "./test_file-created_from_str_ref";
        let _test_file = TestFile::from(filename);
    }

    #[test]
    fn created_from_string() {
        let filename = "./test_file-created_from_string".to_string();
        let _test_file = TestFile::from(filename);
    }

    #[test]
    fn existing_file_is_deleted_on_construction() {
        let filename = "./test_file-existing_file_is_deleted_on_construction";
        ensure_file(filename);
        let _test_file = TestFile::from(filename);
        assert!(!Path::new(filename).exists());
    }

    #[test]
    fn file_is_deleted_on_destruction() {
        let filename = "./test_file-file_is_deleted_on_destruction";

        {
            let _test_file = TestFile::from(filename);
            ensure_file(filename);
        }

        assert!(!Path::new(filename).exists());
    }

    #[test]
    fn get_file_name() {
        let filename = "./test_file-get_file_name";
        let test_file = TestFile::from(filename);

        assert_eq!(test_file.file_name(), filename);
    }

    #[test]
    fn hidden_file_is_deleted_on_construction() {
        let filename = "./test_file-hidden_file_is_deleted_on_construction";
        let hidden_filename = "./.test_file-hidden_file_is_deleted_on_construction";
        ensure_file(filename);
        ensure_file(hidden_filename);
        let _test_file = TestFile::from(filename);
        assert!(!Path::new(filename).exists());
        assert!(!Path::new(hidden_filename).exists());
    }

    #[test]
    fn hidden_file_is_deleted_on_destruction() {
        let filename = "test_file-hidden_file_is_deleted_on_destruction";
        let hidden_filename = ".test_file-hidden_file_is_deleted_on_destruction";

        {
            let _test_file = TestFile::from(filename);
            ensure_file(filename);
            ensure_file(hidden_filename);
        }

        assert!(!Path::new(filename).exists());
        assert!(!Path::new(hidden_filename).exists());
    }

    #[test]
    fn new() {
        let caller = Location::caller();
        let current_source_file = Path::new(caller.file())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let test_file = TestFile::new();
        assert!(!test_file.file_name().is_empty());
        assert!(test_file.file_name().contains(current_source_file));
    }
}
