use crate::bench_result::BenchResult;
use crate::utilities::format_size;
use crate::BENCH_DATABASE;
use crate::CELL_PADDING;
use crate::PADDING;
use agdb::Db;
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) struct Database(pub(crate) Arc<RwLock<Db>>);

impl Database {
    pub(crate) fn new() -> BenchResult<Self> {
        remove_db_files();
        let db = Db::new(BENCH_DATABASE)?;
        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) fn stat(&mut self) -> BenchResult<()> {
        let original_size = std::fs::metadata(BENCH_DATABASE)?.len();

        self.0.write()?.optimize_storage()?;

        let db_size = std::fs::metadata(BENCH_DATABASE)?.len();

        println!(
            "{:PADDING$} | {:CELL_PADDING$} | {} (optimized)",
            "Database size",
            format_size(original_size),
            format_size(db_size)
        );
        Ok(())
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        remove_db_files()
    }
}

fn remove_db_files() {
    let _ = std::fs::remove_file(format!(".{BENCH_DATABASE}"));
    let _ = std::fs::remove_file(BENCH_DATABASE);
}
