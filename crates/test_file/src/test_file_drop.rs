use crate::TestFile;

impl Drop for TestFile {
    fn drop(&mut self) {
        Self::remove_file_if_exists(&self.filename);
        Self::remove_file_if_exists(&Self::hidden_filename(&self.filename));
    }
}
