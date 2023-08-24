use crate::bench_result::BenchResult;
use crate::BENCH_DATABASE;
use agdb::Db;
use num_format::Locale;
use num_format::ToFormattedString;
use std::os::windows::prelude::MetadataExt;
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) struct Database(pub(crate) Arc<RwLock<Db>>);

impl Database {
    pub(crate) fn new() -> BenchResult<Self> {
        remove_db_file();
        let db = Db::new(BENCH_DATABASE)?;
        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) fn stat() -> BenchResult<()> {
        let db_size = std::fs::metadata(BENCH_DATABASE)?.file_size();
        println!(
            "Database size: {} bytes",
            db_size.to_formatted_string(&Locale::en)
        );
        Ok(())
    }
}

fn remove_db_file() {
    let _ = std::fs::remove_file(format!(".{BENCH_DATABASE}"));
    let _ = std::fs::remove_file(BENCH_DATABASE);
}
