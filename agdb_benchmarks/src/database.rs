use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::utilities::format_size;
use agdb::DbImpl;
use agdb::QueryBuilder;
use agdb::StorageData;
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) struct Database<S: StorageData>(pub(crate) Arc<RwLock<DbImpl<S>>>);

impl<S: StorageData> Database<S> {
    pub(crate) fn new(config: &Config) -> BenchResult<Self> {
        remove_db_files(&config.db_name);
        let mut db = DbImpl::new(&config.db_name)?;
        db.exec_mut(QueryBuilder::insert().nodes().aliases("users").query())?;
        db.exec_mut(QueryBuilder::insert().nodes().aliases("posts").query())?;
        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) fn stat(&mut self, config: &Config) -> BenchResult<()> {
        let original_size = self.0.read()?.size();

        self.0.write()?.optimize_storage()?;

        let db_size = self.0.read()?.size();

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

impl<S: StorageData> Clone for Database<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

fn remove_db_files(db_name: &str) {
    let path = std::path::PathBuf::from(format!(".{db_name}"));
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    let path = std::path::PathBuf::from(db_name);
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}
