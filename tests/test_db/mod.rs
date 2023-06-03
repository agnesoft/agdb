#[path = "../../src/agdb/test_utilities/test_file.rs"]
mod test_file;

use agdb::Db;
use agdb::DbElement;
use agdb::Query;
use agdb::QueryError;
use agdb::QueryMut;
use agdb::TransactionMut;
use test_file::TestFile;

pub struct TestDb {
    _test_file: TestFile,
    pub db: Db,
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
    pub fn exec<T: Query>(&self, query: T, result: i64) {
        assert_eq!(self.db.exec(&query).unwrap().result, result);
    }

    #[track_caller]
    pub fn exec_ids<T: Query>(&mut self, query: T, ids: &[i64]) {
        assert_eq!(
            self.db
                .exec(&query)
                .unwrap()
                .elements
                .into_iter()
                .map(|e| e.index.0)
                .collect::<Vec<i64>>(),
            ids
        );
    }

    #[track_caller]
    pub fn exec_elements<T: Query>(&mut self, query: T, elements: &[DbElement]) {
        let res = self.db.exec(&query).unwrap();
        println!("RESULT: {}", res.result);
        assert_eq!(res.result, elements.len() as i64);
        assert_eq!(res.elements, elements);
    }

    #[track_caller]
    pub fn exec_error<T: Query>(&self, query: T, error: &str) {
        assert_eq!(self.db.exec(&query).unwrap_err().description, error);
    }

    #[track_caller]
    pub fn exec_mut<T: QueryMut>(&mut self, query: T, result: i64) {
        assert_eq!(self.db.exec_mut(&query).unwrap().result, result);
    }

    #[track_caller]
    pub fn exec_mut_ids<T: QueryMut>(&mut self, query: T, ids: &[i64]) {
        assert_eq!(
            self.db
                .exec_mut(&query)
                .unwrap()
                .elements
                .into_iter()
                .map(|e| e.index.0)
                .collect::<Vec<i64>>(),
            ids
        );
    }

    #[track_caller]
    pub fn exec_mut_error<T: QueryMut>(&mut self, query: T, error: &str) {
        assert_eq!(self.db.exec_mut(&query).unwrap_err().description, error);
    }

    #[track_caller]
    pub fn transaction_mut<T, E: From<QueryError> + std::fmt::Debug>(
        &mut self,
        f: impl Fn(&mut TransactionMut) -> Result<T, E>,
    ) {
        self.db.transaction_mut(f).unwrap();
    }

    #[track_caller]
    pub fn transaction_mut_error<
        T: std::fmt::Debug,
        E: From<QueryError> + std::fmt::Debug + PartialEq,
    >(
        &mut self,
        f: impl Fn(&mut TransactionMut) -> Result<T, E>,
        error: E,
    ) {
        assert_eq!(self.db.transaction_mut(f).unwrap_err(), error);
    }
}
