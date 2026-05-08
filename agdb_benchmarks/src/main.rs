use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::StorageData;
use bench_result::BenchResult;
use config::BenchmarkMode;
use config::Config;
use config::DbType;
use database::Database;
use database::ServerDatabase;

use crate::utilities::print_flush;

mod bench_error;
mod bench_result;
mod config;
mod database;
mod queries;
mod readers;
mod users;
mod utilities;
mod writers;

pub(crate) const BENCH_CONFIG_FILE: &str = "agdb_benchmarks.yaml";

async fn benchmark<S: StorageData + Send + Sync + 'static>(config: &Config) -> BenchResult<()> {
    let mut db = Database::<S>::new(config)?;
    users::setup_users(&mut db, config)?;

    let mut posters = writers::start_post_writers(&mut db, config)?;
    let mut commenters = writers::start_comment_writers(&mut db, config)?;
    let mut post_readers = readers::start_post_readers(&mut db, config)?;
    let mut comment_readers = readers::start_comment_readers(&mut db, config)?;

    posters
        .join_and_report(
            "Write posts",
            config.posters.count,
            config.posters.posts,
            1,
            config,
        )
        .await?;
    commenters
        .join_and_report(
            "Write comments",
            config.commenters.count,
            config.commenters.comments,
            1,
            config,
        )
        .await?;
    post_readers
        .join_and_report(
            "Read posts",
            config.post_readers.count,
            config.post_readers.reads_per_reader,
            config.post_readers.posts,
            config,
        )
        .await?;
    comment_readers
        .join_and_report(
            "Read comments",
            config.comment_readers.count,
            config.comment_readers.reads_per_reader,
            config.comment_readers.comments,
            config,
        )
        .await?;

    println!("---");
    db.stat(config)
}

async fn benchmark_embedded(config: &Config) -> BenchResult<()> {
    match config.db_type {
        DbType::File => benchmark::<FileStorage>(config).await,
        DbType::FileMapped => benchmark::<FileStorageMemoryMapped>(config).await,
        DbType::InMemory => benchmark::<MemoryStorage>(config).await,
    }
}

async fn benchmark_server(config: &Config, address: &str) -> BenchResult<()> {
    let mut db = ServerDatabase::new(config, address).await?;
    users::setup_server_users(&mut db, config).await?;
    users::setup_server_bench_users(&db, config).await?;

    let mut posters = writers::start_post_writers_server(&db, config).await?;
    let mut commenters = writers::start_comment_writers_server(&db, config).await?;
    let mut post_readers = readers::start_post_readers_server(&db, config).await?;
    let mut comment_readers = readers::start_comment_readers_server(&db, config).await?;

    posters
        .join_and_report(
            "Write posts",
            config.posters.count,
            config.posters.posts,
            1,
            config,
        )
        .await?;
    commenters
        .join_and_report(
            "Write comments",
            config.commenters.count,
            config.commenters.comments,
            1,
            config,
        )
        .await?;
    post_readers
        .join_and_report(
            "Read posts",
            config.post_readers.count,
            config.post_readers.reads_per_reader,
            config.post_readers.posts,
            config,
        )
        .await?;
    comment_readers
        .join_and_report(
            "Read comments",
            config.comment_readers.count,
            config.comment_readers.reads_per_reader,
            config.comment_readers.comments,
            config,
        )
        .await?;

    db.stat(config).await
}

#[tokio::main]
async fn main() -> BenchResult<()> {
    let config = Config::load_config()?;

    print_flush("Running agdb benchmark\n---\n\n".to_string());
    utilities::print_header(&config);

    match &config.mode {
        BenchmarkMode::Embedded => benchmark_embedded(&config).await,
        BenchmarkMode::Server { address } => benchmark_server(&config, address).await,
    }
}
