use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::utilities::format_size;
use agdb::Db;
use agdb::QueryBuilder;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Clone)]
pub(crate) struct Database(pub(crate) Arc<RwLock<Db>>);

impl Database {
    pub(crate) fn new(config: &Config) -> BenchResult<Self> {
        remove_db_files(&config.db_name);
        let mut db = Db::new(&config.db_name)?;
        db.exec_mut(&QueryBuilder::insert().nodes().aliases("users").query())?;
        db.exec_mut(&QueryBuilder::insert().nodes().aliases("posts").query())?;
        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) fn stat(&mut self, config: &Config) -> BenchResult<()> {
        let original_size = std::fs::metadata(&config.db_name)?.len();

        self.0.write()?.optimize_storage()?;

        let db_size = std::fs::metadata(&config.db_name)?.len();

        let padding = config.padding as usize;
        let cell_padding = config.cell_padding as usize;

        println!(
            "{:padding$} | {:cell_padding$} | {} (optimized)",
            "Database size",
            format_size(original_size, config.locale),
            format_size(db_size, config.locale)
        );

        Ok(())
    }
}

fn remove_db_files(db_name: &str) {
    let _ = std::fs::remove_file(format!(".{db_name}"));
    let _ = std::fs::remove_file(db_name);
}
