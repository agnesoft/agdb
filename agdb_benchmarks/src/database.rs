use crate::bench_result::BenchResult;
use agdb::Db;
use std::sync::Arc;
use std::sync::RwLock;

const BENCH_DATABASE: &str = "agdb.benchmark";

pub(crate) struct Database(pub(crate) Arc<RwLock<Db>>);

impl Database {
    pub fn new() -> BenchResult<Self> {
        remove_db_file();
        let db = Db::new(BENCH_DATABASE)?;
        Ok(Self(Arc::new(RwLock::new(db))))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        remove_db_file();
    }
}

fn remove_db_file() {
    let _ = std::fs::remove_file(format!(".{BENCH_DATABASE}"));
    let _ = std::fs::remove_file(BENCH_DATABASE);
}
