use crate::action::ClusterAction;
use crate::cluster::Cluster;
use crate::cluster_log::CLUSTER_LOG_FILE;
use crate::config::Config;
use crate::db_pool;
use crate::db_pool::DbName;
use crate::raft::Log;
use crate::server_db::SERVER_DB_FILE;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use crate::user_id::ClusterId;
use agdb_api::DbKind;
use axum::body::Body;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use crc::CRC_64_XZ;
use crc::Crc;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
const SNAPSHOT_STAGING_TTL_SECS: u64 = 600;

struct SnapshotInFlightGuard {
    counter: Arc<AtomicUsize>,
    /// Optional staging dir to remove when the stream completes.
    staging_cleanup: Option<PathBuf>,
}

impl Drop for SnapshotInFlightGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Release);
        if let Some(dir) = self.staging_cleanup.take() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

// ---------------------------------------------------------------------------
// Full-cluster snapshot endpoint
// ---------------------------------------------------------------------------

pub(crate) async fn snapshot(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
    headers: axum::http::HeaderMap,
) -> ServerResult<axum::response::Response> {
    if cluster.resync.load(Ordering::Acquire) {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("resyncing"))
            .map_err(|e| ServerError::from(e.to_string()));
    }

    let data_dir = std::path::Path::new(&config.data_dir);

    let (_, log_term_curr, log_commit_curr) = {
        let raft = cluster.raft.read().await;
        (raft.storage.index, raft.storage.term, raft.storage.commit)
    };

    if cluster.snapshot_in_flight.load(Ordering::Acquire) == 0 {
        cleanup_stale_snapshot_stagings(data_dir, log_commit_curr, log_term_curr);
    }

    cluster.snapshot_in_flight.fetch_add(1, Ordering::Release);

    let staging_dir = match ensure_snapshot_staged(&cluster, &config).await {
        Ok(d) => d,
        Err(e) => {
            cluster.snapshot_in_flight.fetch_sub(1, Ordering::Release);
            return Err(e);
        }
    };

    let snapshot_meta = std::fs::read_to_string(staging_dir.join(".id")).unwrap_or_default();
    let snapshot_meta = snapshot_meta.trim().to_string();
    let (log_index, log_term, log_commit) = match parse_snapshot_meta(&snapshot_meta) {
        Ok(v) => v,
        Err(e) => {
            cluster.snapshot_in_flight.fetch_sub(1, Ordering::Release);
            return Err(e);
        }
    };

    let resume_id = headers
        .get("x-snapshot-resume-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let start_offset = if resume_id == snapshot_meta && !resume_id.is_empty() {
        headers
            .get("range")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("bytes="))
            .and_then(|s| s.trim_end_matches('-').parse::<u64>().ok())
            .unwrap_or(0)
    } else {
        0
    };

    let is_resume = start_offset > 0;
    let status = if is_resume {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };

    let body = match snapshot_body_stream(
        staging_dir,
        SnapshotStreamParams {
            start_offset,
            log_index,
            log_term,
            log_commit,
            in_flight: cluster.snapshot_in_flight.clone(),
            chunk_size: config.cluster_max_chunk_size as usize,
            staging_cleanup: None, // full-cluster staging dir is reusable
        },
    ) {
        Ok(b) => b,
        Err(e) => {
            cluster.snapshot_in_flight.fetch_sub(1, Ordering::Release);
            return Err(e);
        }
    };

    axum::response::Response::builder()
        .status(status)
        .header("content-type", "application/octet-stream")
        .header("x-snapshot-id", &snapshot_meta)
        .body(body)
        .map_err(|e| ServerError::from(e.to_string()))
}

fn parse_snapshot_meta(meta: &str) -> ServerResult<(u64, u64, u64)> {
    let parts: Vec<u64> = meta.split('_').filter_map(|s| s.parse().ok()).collect();
    match parts.as_slice() {
        [log_index, log_term, log_commit] => Ok((*log_index, *log_term, *log_commit)),
        _ => Err(ServerError::from(format!(
            "malformed snapshot .id: {meta:?}"
        ))),
    }
}

async fn ensure_snapshot_staged(cluster: &Cluster, config: &Config) -> ServerResult<PathBuf> {
    let data_dir = std::path::Path::new(&config.data_dir);
    let (log_index, log_term, log_commit) = {
        let raft = cluster.raft.read().await;
        (raft.storage.index, raft.storage.term, raft.storage.commit)
    };

    let staging_dir = data_dir.with_file_name(format!(".snapshot_staging_{log_commit}_{log_term}"));

    if staging_dir.join(SERVER_DB_FILE).exists() {
        return Ok(staging_dir);
    }

    if staging_dir.exists() {
        let _ = std::fs::remove_dir_all(&staging_dir);
    }
    std::fs::create_dir_all(&staging_dir)?;

    if let Err(e) = stage_snapshot_files(
        cluster,
        config,
        &staging_dir,
        log_index,
        log_term,
        log_commit,
    )
    .await
    {
        let _ = std::fs::remove_dir_all(&staging_dir);
        return Err(e);
    }

    Ok(staging_dir)
}

async fn stage_snapshot_files(
    cluster: &Cluster,
    config: &Config,
    staging_dir: &std::path::Path,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
) -> ServerResult<()> {
    let data_dir = std::path::Path::new(&config.data_dir);

    let (snapshot_lock_arc, server_db, cluster_log_ref, db_pool_ref) = {
        let raft = cluster.raft.read().await;
        (
            raft.storage.snapshot_lock.clone(),
            raft.storage.db.clone(),
            raft.storage.cluster_log.clone(),
            raft.storage.db_pool.clone(),
        )
    };
    let _snapshot_lock = snapshot_lock_arc.write_owned().await;

    let dbs = server_db.dbs().await?;

    let db_inner = server_db.db.clone();
    tokio::task::spawn_blocking(move || db_inner.blocking_write().sync()).await??;

    let log_inner = cluster_log_ref.0.clone();
    tokio::task::spawn_blocking(move || log_inner.blocking_write().sync()).await??;

    let db_entries: Vec<_> = {
        let pool = db_pool_ref.pool.read().await;
        dbs.iter()
            .filter_map(|db_info| {
                let db_key = DbName {
                    owner: db_info.owner.clone(),
                    db: db_info.db.clone(),
                };
                pool.get(&db_key).map(|user_db| {
                    (
                        db_info.owner.clone(),
                        db_info.db.clone(),
                        db_info.db_type,
                        user_db.clone(),
                    )
                })
            })
            .collect()
    };

    for (owner, db_name, db_type, user_db) in &db_entries {
        let db_src = db_pool::db_file(owner, db_name, config);
        let db_rel = db_src
            .strip_prefix(data_dir)
            .map_err(|e| ServerError::from(e.to_string()))?;
        let db_staging = staging_dir.join(db_rel);

        if let Some(parent) = db_staging.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if *db_type == DbKind::Memory {
            let db_arc = user_db.0.clone();
            let staging_path = db_staging
                .to_str()
                .ok_or_else(|| ServerError::from("invalid staging path"))?
                .to_string();
            tokio::task::spawn_blocking(move || db_arc.blocking_read().backup(&staging_path))
                .await??;
        } else {
            let db_arc = user_db.0.clone();
            let staging_path = db_staging
                .to_str()
                .ok_or_else(|| ServerError::from("invalid staging path"))?
                .to_string();
            tokio::task::spawn_blocking(move || {
                let mut guard = db_arc.blocking_write();
                guard.sync()?;
                guard.backup(&staging_path)
            })
            .await??;
        }

        for src in [
            db_pool::db_audit_file(owner, db_name, config),
            db_pool::db_backup_file(owner, db_name, config),
            db_pool::db_backup_audit_file(owner, db_name, config),
        ] {
            if src.exists() {
                let rel = src
                    .strip_prefix(data_dir)
                    .map_err(|e| ServerError::from(e.to_string()))?;
                let dst = staging_dir.join(rel);
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                tokio::fs::copy(&src, &dst).await?;
            }
        }
    }

    let cluster_log_staging_str = staging_dir
        .join(CLUSTER_LOG_FILE)
        .to_str()
        .ok_or_else(|| ServerError::from("invalid staging path"))?
        .to_string();
    let log_inner = cluster_log_ref.0.clone();
    tokio::task::spawn_blocking(move || log_inner.blocking_read().backup(&cluster_log_staging_str))
        .await??;

    let server_db_tmp = staging_dir.join(".server_db_tmp");
    let server_db_tmp_str = server_db_tmp
        .to_str()
        .ok_or_else(|| ServerError::from("invalid staging path"))?
        .to_string();
    let db_inner = server_db.db.clone();
    tokio::task::spawn_blocking(move || db_inner.blocking_read().backup(&server_db_tmp_str))
        .await??;
    std::fs::rename(&server_db_tmp, staging_dir.join(SERVER_DB_FILE))?;

    std::fs::write(
        staging_dir.join(".id"),
        format!("{log_index}_{log_term}_{log_commit}"),
    )?;

    Ok(())
}

struct SnapshotStreamParams {
    start_offset: u64,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
    in_flight: Arc<AtomicUsize>,
    chunk_size: usize,
    staging_cleanup: Option<PathBuf>,
}

fn snapshot_body_stream(staging_dir: PathBuf, params: SnapshotStreamParams) -> ServerResult<Body> {
    let files = list_snapshot_files(&staging_dir)?;
    Ok(Body::from_stream(snapshot_file_stream(files, params)))
}

fn snapshot_file_stream(
    files: Vec<(String, PathBuf)>,
    params: SnapshotStreamParams,
) -> impl futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send + 'static {
    let file_count = files.len() as u64;
    let guard = SnapshotInFlightGuard {
        counter: params.in_flight.clone(),
        staging_cleanup: params.staging_cleanup,
    };
    let SnapshotStreamParams {
        start_offset,
        log_index,
        log_term,
        log_commit,
        chunk_size,
        ..
    } = params;
    let compute_crc = start_offset == 0;

    async_stream::try_stream! {
        let _guard = guard;

        let crc64 = Crc::<u64>::new(&CRC_64_XZ);
        let mut digest = crc64.digest();
        let mut pos = 0u64;

        let mut header = [0u8; 32];
        header[..8].copy_from_slice(&log_index.to_le_bytes());
        header[8..16].copy_from_slice(&log_term.to_le_bytes());
        header[16..24].copy_from_slice(&log_commit.to_le_bytes());
        header[24..].copy_from_slice(&file_count.to_le_bytes());

        if 32u64 > start_offset {
            let skip = start_offset.saturating_sub(pos) as usize;
            let chunk = &header[skip..];
            if compute_crc { digest.update(chunk); }
            yield bytes::Bytes::copy_from_slice(chunk);
        }
        pos = 32;

        let mut buf = vec![0u8; chunk_size];

        for (rel_path, abs_path) in files {
            use tokio::io::AsyncReadExt;
            use tokio::io::AsyncSeekExt;

            let path_bytes = rel_path.as_bytes();
            let file_len = tokio::fs::metadata(&abs_path).await?.len();

            let mut meta = Vec::with_capacity(4 + path_bytes.len() + 8);
            meta.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
            meta.extend_from_slice(path_bytes);
            meta.extend_from_slice(&file_len.to_le_bytes());

            let meta_end = pos + meta.len() as u64;
            if meta_end > start_offset {
                let skip = start_offset.saturating_sub(pos) as usize;
                let chunk = &meta[skip..];
                if compute_crc { digest.update(chunk); }
                yield bytes::Bytes::copy_from_slice(chunk);
            }
            pos = meta_end;

            let data_end = pos + file_len;
            if data_end > start_offset {
                let seek_to = start_offset.saturating_sub(pos);
                let mut f = tokio::fs::File::open(&abs_path).await?;
                if seek_to > 0 {
                    f.seek(std::io::SeekFrom::Start(seek_to)).await?;
                }
                loop {
                    let n = f.read(&mut buf).await?;
                    if n == 0 {
                        break;
                    }
                    if compute_crc { digest.update(&buf[..n]); }
                    yield bytes::Bytes::copy_from_slice(&buf[..n]);
                }
            }
            pos = data_end;
        }

        if compute_crc {
            let checksum = digest.finalize();
            yield bytes::Bytes::copy_from_slice(&checksum.to_le_bytes());
        }
    }
}

fn list_snapshot_files(staging_dir: &std::path::Path) -> ServerResult<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    collect_staging_files(staging_dir, staging_dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn collect_staging_files(
    dir: &std::path::Path,
    base: &std::path::Path,
    files: &mut Vec<(String, PathBuf)>,
) -> ServerResult<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_staging_files(&path, base, files)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| ServerError::from(e.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((rel, path));
        }
    }
    Ok(())
}

fn cleanup_stale_snapshot_stagings(
    data_dir: &std::path::Path,
    current_commit: u64,
    current_term: u64,
) {
    let Some(parent) = data_dir.parent() else {
        return;
    };

    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };

    let now = std::time::SystemTime::now();
    let current_suffix = format!("{current_commit}_{current_term}");

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(".snapshot_staging_") {
            continue;
        }
        // Skip the staging dir matching current raft state (may be serving or about to be built)
        let suffix = name_str.strip_prefix(".snapshot_staging_").unwrap_or("");
        if suffix == current_suffix {
            continue;
        }
        if !entry.path().is_dir() {
            continue;
        }
        if let Ok(metadata) = entry.metadata()
            && let Ok(modified) = metadata.modified()
            && let Ok(age) = now.duration_since(modified)
            && age.as_secs() >= SNAPSHOT_STAGING_TTL_SECS
        {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

// ---------------------------------------------------------------------------
// Log catch-up endpoint
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub(crate) struct LogsQuery {
    from_index: u64,
}

pub(crate) async fn logs(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
    Query(params): Query<LogsQuery>,
) -> ServerResult<axum::response::Response> {
    if cluster.resync.load(Ordering::Acquire) {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("resyncing"))
            .map_err(|e| ServerError::from(e.to_string()));
    }

    let (cluster_log, commit_index) = {
        let raft = cluster.raft.read().await;
        (raft.storage.cluster_log.clone(), raft.storage.commit)
    };
    let entries = cluster_log.logs_since(params.from_index).await?;

    if entries.is_empty() {
        return axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("no entries available"))
            .map_err(|e| ServerError::from(e.to_string()));
    }

    let chunk_size = config.cluster_max_chunk_size as usize;
    let body = Body::from_stream(log_entries_byte_stream(entries, commit_index, chunk_size));

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/octet-stream")
        .body(body)
        .map_err(|e| ServerError::from(e.to_string()))
}

fn log_entries_byte_stream(
    logs: Vec<Log<ClusterAction>>,
    commit_index: u64,
    chunk_size: usize,
) -> impl futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send + 'static {
    async_stream::try_stream! {
        let entry_count = logs.len() as u64;

        let mut header = [0u8; 16];
        header[0..8].copy_from_slice(&entry_count.to_le_bytes());
        header[8..16].copy_from_slice(&commit_index.to_le_bytes());
        yield bytes::Bytes::copy_from_slice(&header);

        let mut buf = Vec::with_capacity(chunk_size);

        for log in &logs {
            let json = serde_json::to_vec(log)
                .map_err(std::io::Error::other)?;
            let len_bytes = (json.len() as u64).to_le_bytes();

            if !buf.is_empty() && buf.len() + 8 + json.len() > chunk_size {
                yield bytes::Bytes::from(std::mem::take(&mut buf));
            }

            buf.extend_from_slice(&len_bytes);
            buf.extend_from_slice(&json);

            if buf.len() >= chunk_size {
                yield bytes::Bytes::from(std::mem::take(&mut buf));
            }
        }
        if !buf.is_empty() {
            yield bytes::Bytes::from(buf);
        }
    }
}

// ---------------------------------------------------------------------------
// Single-DB snapshot endpoints
// ---------------------------------------------------------------------------

pub(crate) async fn snapshot_db(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
    State(server_db): State<ServerDb>,
    Path((owner, db)): Path<(String, String)>,
) -> ServerResult<axum::response::Response> {
    if cluster.resync.load(Ordering::Acquire) {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("resyncing"))
            .map_err(|e| ServerError::from(e.to_string()));
    }

    let snapshot_lock_arc = cluster.raft.read().await.storage.snapshot_lock.clone();
    let _snapshot_lock = snapshot_lock_arc.read().await;

    let data_dir = std::path::Path::new(&config.data_dir);
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging_dir = std::env::temp_dir().join(format!(
        "agdb_snapshot_db_{}_{}_{}_{}",
        owner,
        db,
        std::process::id(),
        unique_id
    ));
    if staging_dir.exists() {
        let _ = std::fs::remove_dir_all(&staging_dir);
    }
    std::fs::create_dir_all(&staging_dir)?;

    let result = stage_single_db(
        &cluster,
        &config,
        &server_db,
        &staging_dir,
        data_dir,
        &owner,
        &db,
    )
    .await;
    if let Err(e) = result {
        let _ = std::fs::remove_dir_all(&staging_dir);
        return Err(e);
    }

    let db_type_str = match server_db
        .dbs()
        .await?
        .into_iter()
        .find(|d| d.owner == owner && d.db == db)
        .map(|d| d.db_type)
    {
        Some(DbKind::Memory) => "memory",
        Some(DbKind::File) => "file",
        _ => "mapped",
    };

    cluster.snapshot_in_flight.fetch_add(1, Ordering::Release);

    let staging_cleanup = staging_dir.clone();
    let body = match snapshot_body_stream(
        staging_dir,
        SnapshotStreamParams {
            start_offset: 0,
            log_index: 0,
            log_term: 0,
            log_commit: 0,
            in_flight: cluster.snapshot_in_flight.clone(),
            chunk_size: config.cluster_max_chunk_size as usize,
            staging_cleanup: Some(staging_cleanup),
        },
    ) {
        Ok(b) => b,
        Err(e) => {
            cluster.snapshot_in_flight.fetch_sub(1, Ordering::Release);
            return Err(e);
        }
    };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/octet-stream")
        .header("x-db-type", db_type_str)
        .body(body)
        .map_err(|e| ServerError::from(e.to_string()))
}

async fn stage_single_db(
    cluster: &Cluster,
    config: &Config,
    server_db: &ServerDb,
    staging_dir: &std::path::Path,
    data_dir: &std::path::Path,
    owner: &str,
    db: &str,
) -> ServerResult<()> {
    let db_pool_ref = cluster.raft.read().await.storage.db_pool.clone();
    let db_key = DbName {
        owner: owner.to_string(),
        db: db.to_string(),
    };

    let user_db = {
        let pool = db_pool_ref.pool.read().await;
        match pool.get(&db_key) {
            Some(db) => db.clone(),
            None => {
                return Err(ServerError {
                    description: "db not found".to_string(),
                    status: StatusCode::NOT_FOUND,
                });
            }
        }
    };

    let db_src = db_pool::db_file(owner, db, config);
    let db_rel = db_src
        .strip_prefix(data_dir)
        .map_err(|e| ServerError::from(e.to_string()))?;
    let db_staging = staging_dir.join(db_rel);

    if let Some(parent) = db_staging.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_type = server_db
        .dbs()
        .await?
        .into_iter()
        .find(|d| d.owner == owner && d.db == db)
        .map(|d| d.db_type)
        .unwrap_or(DbKind::Mapped);

    let staging_path = db_staging
        .to_str()
        .ok_or_else(|| ServerError::from("invalid staging path"))?
        .to_string();

    if db_type == DbKind::Memory {
        let db_arc = user_db.0.clone();
        tokio::task::spawn_blocking(move || db_arc.blocking_read().backup(&staging_path)).await??;
    } else {
        let db_arc = user_db.0.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = db_arc.blocking_write();
            guard.sync()?;
            guard.backup(&staging_path)
        })
        .await??;
    }

    for src in [
        db_pool::db_audit_file(owner, db, config),
        db_pool::db_backup_file(owner, db, config),
        db_pool::db_backup_audit_file(owner, db, config),
    ] {
        if src.exists() {
            let rel = src
                .strip_prefix(data_dir)
                .map_err(|e| ServerError::from(e.to_string()))?;
            let dst = staging_dir.join(rel);
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            tokio::fs::copy(&src, &dst).await?;
        }
    }

    Ok(())
}

pub(crate) async fn snapshot_server_db(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
    State(server_db): State<ServerDb>,
) -> ServerResult<axum::response::Response> {
    if cluster.resync.load(Ordering::Acquire) {
        return axum::response::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("resyncing"))
            .map_err(|e| ServerError::from(e.to_string()));
    }

    let snapshot_lock_arc = cluster.raft.read().await.storage.snapshot_lock.clone();
    let _snapshot_lock = snapshot_lock_arc.read().await;

    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging_dir = std::env::temp_dir().join(format!(
        "agdb_snapshot_server_db_{}_{}",
        std::process::id(),
        unique_id
    ));
    if staging_dir.exists() {
        let _ = std::fs::remove_dir_all(&staging_dir);
    }
    std::fs::create_dir_all(&staging_dir)?;

    let server_db_staging = staging_dir.join(SERVER_DB_FILE);
    let staging_path = server_db_staging
        .to_str()
        .ok_or_else(|| ServerError::from("invalid staging path"))?
        .to_string();

    let db_arc = server_db.db.clone();
    tokio::task::spawn_blocking(move || db_arc.blocking_read().backup(&staging_path)).await??;

    cluster.snapshot_in_flight.fetch_add(1, Ordering::Release);

    let staging_cleanup = staging_dir.clone();
    let body = match snapshot_body_stream(
        staging_dir,
        SnapshotStreamParams {
            start_offset: 0,
            log_index: 0,
            log_term: 0,
            log_commit: 0,
            in_flight: cluster.snapshot_in_flight.clone(),
            chunk_size: config.cluster_max_chunk_size as usize,
            staging_cleanup: Some(staging_cleanup),
        },
    ) {
        Ok(b) => b,
        Err(e) => {
            cluster.snapshot_in_flight.fetch_sub(1, Ordering::Release);
            return Err(e);
        }
    };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/octet-stream")
        .body(body)
        .map_err(|e| ServerError::from(e.to_string()))
}
