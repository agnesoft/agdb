use crate::TestFile;

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
