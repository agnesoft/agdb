use crate::test_server::TestServerImpl;
use crate::test_server::test_error::TestError;
use std::path::PathBuf;

pub struct TestDir {
    pub dir: PathBuf,
}

impl TestDir {
    pub fn new() -> Result<Self, TestError> {
        let dir = format!("static_files_test{}", TestServerImpl::next_port()).into();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
