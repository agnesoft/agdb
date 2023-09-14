use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::StorageData;
use bench_result::BenchResult;
use config::Config;
use config::DbType;
use database::Database;

mod bench_error;
mod bench_result;
mod config;
mod database;
mod readers;
mod users;
mod utilities;
mod writers;

pub(crate) const BENCH_CONFIG_FILE: &str = "agdb_benchmarks.yaml";

async fn benchmark<S: StorageData + Send + Sync + 'static>(config: &Config) -> BenchResult<()> {
    let mut db = Database::<S>::new(&config)?;
    users::setup_users(&mut db, &config)?;

    let mut posters = writers::start_post_writers(&mut db, &config)?;
    let mut commenters = writers::start_comment_writers(&mut db, &config)?;
    let mut post_readers = readers::start_post_readers(&mut db, &config)?;
    let mut comment_readers = readers::start_comment_readers(&mut db, &config)?;

    posters
        .join_and_report(
            "Write posts",
            config.posters.count,
            config.posters.posts,
            1,
            &config,
        )
        .await?;
    commenters
        .join_and_report(
            "Write comments",
            config.commenters.count,
            config.commenters.comments,
            1,
            &config,
        )
        .await?;
    post_readers
        .join_and_report(
            "Read posts",
            config.post_readers.count,
            config.post_readers.reads_per_reader,
            config.post_readers.posts,
            &config,
        )
        .await?;
    comment_readers
        .join_and_report(
            "Read comments",
            config.comment_readers.count,
            config.comment_readers.reads_per_reader,
            config.comment_readers.comments,
            &config,
        )
        .await?;

    println!("---");
    db.stat(&config)
}

#[tokio::main]
async fn main() -> BenchResult<()> {
    let config = Config::load_config()?;

    println!("Running agdb benchmark\n\n");
    utilities::print_header(&config);

    match config.db_type {
        DbType::File => benchmark::<FileStorage>(&config).await,
        DbType::FileMapped => benchmark::<FileStorageMemoryMapped>(&config).await,
        DbType::InMemory => benchmark::<MemoryStorage>(&config).await,
    }
}
