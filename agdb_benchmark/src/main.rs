use crate::config::BENCH_CONFIG_FILE;
use crate::config::ServerTargetConfig;
use crate::memory_monitor::ServerMemoryMonitor;
use crate::results::TargetKind;
use crate::results::TargetResult;
use crate::results::WorkloadStats;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::StorageData;
use bench_result::BenchResult;
use config::Config;
use config::DbType;
use database::Database;
use database::DatabaseSize;
use database::ServerDatabase;
use database::create_server_http_client;
use std::time::Instant;

mod bench_error;
mod bench_result;
mod config;
mod database;
mod memory_monitor;
mod queries;
mod readers;
mod results;
mod retry;
mod users;
mod utilities;
mod writers;

async fn benchmark<S: StorageData + Send + Sync + 'static>(
    config: &Config,
) -> BenchResult<(DatabaseSize, WorkloadStats)> {
    let mut db = Database::<S>::new(config)?;
    users::setup_users(&mut db, config)?;

    let mut posters = writers::start_post_writers(&mut db, config)?;
    let mut commenters = writers::start_comment_writers(&mut db, config)?;
    let mut post_readers = readers::start_post_readers(&mut db, config)?;
    let mut comment_readers = readers::start_comment_readers(&mut db, config)?;

    let post_stats = posters.join().await?;
    let comment_stats = commenters.join().await?;
    let post_reader_stats = post_readers.join().await?;
    let comment_reader_stats = comment_readers.join().await?;

    Ok((
        db.stat(config)?,
        WorkloadStats {
            post_writers: post_stats,
            comment_writers: comment_stats,
            post_readers: post_reader_stats,
            comment_readers: comment_reader_stats,
        },
    ))
}

async fn benchmark_embedded(config: &Config) -> BenchResult<(DatabaseSize, WorkloadStats)> {
    match config.db_type {
        DbType::File => benchmark::<FileStorage>(config).await,
        DbType::FileMapped => benchmark::<FileStorageMemoryMapped>(config).await,
        DbType::InMemory => benchmark::<MemoryStorage>(config).await,
    }
}

async fn benchmark_server(
    config: &Config,
    target: &ServerTargetConfig,
) -> BenchResult<(DatabaseSize, WorkloadStats, crate::results::MemoryStats)> {
    let client = create_server_http_client(config)?;
    let mut db = ServerDatabase::new(
        config,
        &target.address,
        &target.admin_username,
        &target.admin_password,
        client.clone(),
    )
    .await?;
    users::setup_server_users(
        &mut db,
        config,
        &target.admin_username,
        &target.admin_password,
    )
    .await?;
    users::setup_server_bench_users(&db, config).await?;
    let monitor = ServerMemoryMonitor::start(
        config,
        &target.address,
        &target.admin_username,
        &target.admin_password,
        &client,
    )
    .await?;

    let mut posters = writers::start_post_writers_server(&db, config).await?;
    let mut commenters = writers::start_comment_writers_server(&db, config).await?;
    let mut post_readers = readers::start_post_readers_server(&db, config).await?;
    let mut comment_readers = readers::start_comment_readers_server(&db, config).await?;

    let post_stats = posters.join().await?;
    let comment_stats = commenters.join().await?;
    let post_reader_stats = post_readers.join().await?;
    let comment_reader_stats = comment_readers.join().await?;

    let db_size = db.stat(config).await?;
    let memory = monitor.finish().await?;

    Ok((
        db_size,
        WorkloadStats {
            post_writers: post_stats,
            comment_writers: comment_stats,
            post_readers: post_reader_stats,
            comment_readers: comment_reader_stats,
        },
        memory,
    ))
}

async fn run_embedded(config: &Config) -> TargetResult {
    let progress = utilities::ProgressIndicator::start("embedded");
    let start = Instant::now();

    let result = match benchmark_embedded(config).await {
        Ok((db_size, workload)) => TargetResult::ok(
            TargetKind::Embedded,
            None,
            start.elapsed(),
            workload,
            Some(db_size.original),
            Some(db_size.optimized),
            None,
        ),
        Err(error) => TargetResult::failed(
            TargetKind::Embedded,
            None,
            start.elapsed(),
            error.description,
        ),
    };

    progress
        .finish(if result.error.is_some() {
            "failed"
        } else {
            "ok"
        })
        .await;

    result
}

async fn run_server(
    config: &Config,
    target: &ServerTargetConfig,
    kind: TargetKind,
) -> TargetResult {
    let progress =
        utilities::ProgressIndicator::start(&format!("{} ({})", kind.as_str(), target.address));
    let start = Instant::now();

    let result = match benchmark_server(config, target).await {
        Ok((db_size, workload, memory)) => TargetResult::ok(
            kind,
            Some(target.address.clone()),
            start.elapsed(),
            workload,
            Some(db_size.original),
            Some(db_size.optimized),
            Some(memory),
        ),
        Err(error) => TargetResult::failed(
            kind,
            Some(target.address.clone()),
            start.elapsed(),
            error.description,
        ),
    };

    progress
        .finish(if result.error.is_some() {
            "failed"
        } else {
            "ok"
        })
        .await;

    result
}

#[tokio::main]
async fn main() -> BenchResult<()> {
    let config_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| BENCH_CONFIG_FILE.to_string());
    let config = Config::new(&config_file)?;

    utilities::print_flush("Running agdb benchmark\n---\n".to_string());

    let mut results = Vec::new();

    if config.targets.embedded {
        results.push(run_embedded(&config).await);
    }

    if let Some(target) = &config.targets.local_server {
        results.push(run_server(&config, target, TargetKind::LocalServer).await);
    }

    if let Some(target) = &config.targets.remote_server {
        results.push(run_server(&config, target, TargetKind::RemoteServer).await);
    }

    if results.is_empty() {
        return Err(crate::bench_error::BenchError {
            description:
                "no benchmark target configured (targets.embedded/local_server/remote_server)"
                    .to_string(),
        });
    }

    utilities::print_final_summary(&config, &results);

    Ok(())
}
