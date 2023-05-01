#[path = "../../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::QueryMut;
use test_file::TestFile;

pub struct TestDb {
    _test_file: TestFile,
    db: Db,
}

#[allow(dead_code)]
impl TestDb {
    #[track_caller]
    pub fn new() -> Self {
        let test_file = TestFile::new();
        let db = Db::new(test_file.file_name()).unwrap();

        Self {
            _test_file: test_file,
            db,
        }
    }

    #[track_caller]
    pub fn exec_mut<T: QueryMut>(&mut self, query: T, result: i64) {
        assert_eq!(self.db.exec_mut(&query).unwrap().result, result);
    }

    #[track_caller]
    pub fn exec_mut_error<T: QueryMut>(&mut self, query: T, error: &str) {
        assert_eq!(self.db.exec_mut(&query).unwrap_err().description, error);
    }
}
