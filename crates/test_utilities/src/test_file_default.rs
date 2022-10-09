use crate::TestFile;

impl Default for TestFile {
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}
