use crate::action::ClusterAction;
use crate::action::remove_all_tokens::RemoveAllTokens;
use crate::action::remove_user_session::RemoveUserSession;
use crate::action::remove_user_token::RemoveUserToken;
use crate::action::remove_user_tokens::RemoveUserTokens;
use crate::action::remove_user_tokens_except::RemoveUserTokensExcept;
use crate::action::save_user_token::SaveUserToken;
use crate::cluster;
use crate::cluster::Cluster;
use crate::cluster_log::CLUSTER_LOG_FILE;
use crate::config::Config;
use crate::db_pool;
use crate::db_pool::DbName;
use crate::raft::Request;
use crate::raft::Response;
use crate::routes::user::LOGOUT_ALL_SESSIONS;
use crate::routes::user::LOGOUT_OTHER_SESSIONS;
use crate::routes::user::LogoutQuery;
use crate::routes::user::do_login;
use crate::server_db::SERVER_DB_FILE;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::server_error::ServerResult;
use crate::user_id::AdminId;
use crate::user_id::ClusterId;
use crate::user_id::UserAgent;
use crate::user_id::UserName;
use crate::user_id::UserToken;
use agdb_api::ClusterStatus;
use agdb_api::DbKind;
use agdb_api::UserLogin;
use axum::Json;
use axum::body::Body;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub(crate) async fn cluster(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    request: Json<Request<ClusterAction>>,
) -> ServerResult<(StatusCode, Json<Response>)> {
    if cluster.resync.load(Ordering::Relaxed) {
        let response = Response {
            target: request.index,
            result: crate::raft::ResponseType::CommitError("resyncing".into()),
        };
        return Ok((StatusCode::OK, Json(response)));
    }

    let response = cluster.raft.write().await.request(&request).await;
    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/admin/user/{username}/logout",
    operation_id = "cluster_admin_user_logout",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
        LogoutQuery,
    ),
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "admin only"),
         (status = 404, description = "user not found"),
    )
)]
pub(crate) async fn admin_logout(
    _admin: AdminId,
    Query(request): Query<LogoutQuery>,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path(username): Path<String>,
) -> ServerResponse<impl IntoResponse> {
    let _user_id = server_db.user_id(&username).await?;

    let (commit_index, _result) = match request.session.as_deref() {
        None | Some(LOGOUT_ALL_SESSIONS) | Some(LOGOUT_OTHER_SESSIONS) => {
            cluster.exec(RemoveUserTokens { user: username }).await?
        }
        Some(session) => {
            cluster
                .exec(RemoveUserSession {
                    session: session.to_string(),
                })
                .await?
        }
    };

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/admin/user/logout_all",
    operation_id = "cluster_admin_user_logout_all",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 201, description = "users logged out"),
         (status = 401, description = "admin only"),
    )
)]
pub(crate) async fn admin_logout_all(
    _admin: AdminId,
    State(cluster): State<Cluster>,
) -> ServerResponse<impl IntoResponse> {
    let (commit_index, _result) = cluster.exec(RemoveAllTokens {}).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/user/login",
    operation_id = "cluster_user_login",
    tag = "agdb",
    request_body = UserLogin,
    responses(
         (status = 200, description = "login successful", body = String),
         (status = 401, description = "invalid credentials"),
    )
)]
pub(crate) async fn login(
    agent: UserAgent,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Json(request): Json<UserLogin>,
) -> ServerResponse<impl IntoResponse> {
    let (_user_id, token, session) =
        do_login(&server_db, &request.username, &request.password).await?;
    let (commit_index, _result) = cluster
        .exec(SaveUserToken {
            user: request.username,
            new_token: token.clone(),
            agent: agent.0,
            session,
        })
        .await?;

    Ok((
        StatusCode::OK,
        [("commit-index", commit_index.to_string())],
        Json(token),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/user/logout",
    operation_id = "cluster_user_logout",
    tag = "agdb",
    security(("Token" = [])),
    params(
        LogoutQuery,
    ),
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "invalid credentials")
    )
)]
pub(crate) async fn logout(
    username: UserName,
    token: UserToken,
    Query(request): Query<LogoutQuery>,
    State(cluster): State<Cluster>,
) -> ServerResponse<impl IntoResponse> {
    let (commit_index, _result) = match request.session.as_deref() {
        None => cluster.exec(RemoveUserToken { token: token.0 }).await?,
        Some(LOGOUT_ALL_SESSIONS) => cluster.exec(RemoveUserTokens { user: username.0 }).await?,
        Some(LOGOUT_OTHER_SESSIONS) => {
            cluster
                .exec(RemoveUserTokensExcept {
                    user: username.0,
                    token: token.0,
                })
                .await?
        }
        Some(session) => {
            cluster
                .exec(RemoveUserSession {
                    session: session.to_string(),
                })
                .await?
        }
    };

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(get,
    path = "/api/v1/cluster/status",
    operation_id = "cluster_status",
    tag = "agdb",
    responses(
         (status = 200, description = "Cluster status", body = Vec<ClusterStatus>),
    )
)]
pub(crate) async fn status(
    State(config): State<Config>,
    State(cluster): State<Cluster>,
) -> ServerResult<(StatusCode, Json<Vec<ClusterStatus>>)> {
    if config.cluster.is_empty() {
        let status = ClusterStatus {
            address: config.server_url(),
            status: true,
            leader: true,
        };
        return Ok((StatusCode::OK, Json(vec![status])));
    }

    let mut statuses = vec![ClusterStatus::default(); config.cluster.len()];
    let mut tasks = Vec::new();
    let leader = cluster.raft.read().await.leader();

    for (index, node) in config.cluster.iter().enumerate() {
        if index != cluster.index {
            let address = node.as_str().to_string();
            let url = format!("{}/api/v1/status", node.trim_end_matches("/"));
            let client = cluster::reqwest_client(&config)?;

            tasks.push(tokio::spawn(async move {
                let response = client
                    .get(&url)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await;

                let status = if let Ok(response) = response {
                    response.status().is_success()
                } else {
                    false
                };

                (
                    index,
                    ClusterStatus {
                        address,
                        status,
                        leader: status && Some(index as u64) == leader,
                    },
                )
            }));
        } else {
            let status = &mut statuses[index];
            status.address = node.as_str().to_string();
            status.status = true;
            status.leader = Some(index as u64) == leader;
        };
    }

    for task in tasks {
        if let Ok((index, status)) = task.await {
            statuses[index] = status;
        }
    }

    statuses.sort_by(|a, b| a.address.cmp(&b.address));

    Ok((StatusCode::OK, Json(statuses)))
}

const SNAPSHOT_STAGING_TTL_SECS: u64 = 600;
const SNAPSHOT_STREAM_CHUNK: usize = 65536;

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
            .map_err(|e| crate::server_error::ServerError::from(e.to_string()));
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
        start_offset,
        log_index,
        log_term,
        log_commit,
        cluster.snapshot_in_flight.clone(),
    ) {
        Ok(b) => b,
        Err(e) => {
            cluster.snapshot_in_flight.fetch_sub(1, Ordering::Release);
            return Err(e);
        }
    };

    let mut builder = axum::response::Response::builder()
        .status(status)
        .header("content-type", "application/octet-stream")
        .header("x-snapshot-id", &snapshot_meta);

    if is_resume {
        builder = builder.header("content-range", format!("bytes {start_offset}-*/*"));
    }

    builder
        .body(body)
        .map_err(|e| crate::server_error::ServerError::from(e.to_string()))
}

fn parse_snapshot_meta(meta: &str) -> ServerResult<(u64, u64, u64)> {
    let parts: Vec<u64> = meta.split('_').filter_map(|s| s.parse().ok()).collect();
    match parts.as_slice() {
        [log_index, log_term, log_commit] => Ok((*log_index, *log_term, *log_commit)),
        _ => Err(crate::server_error::ServerError::from(format!(
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
            .map_err(|e| crate::server_error::ServerError::from(e.to_string()))?;
        let db_staging = staging_dir.join(db_rel);

        if let Some(parent) = db_staging.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if *db_type == DbKind::Memory {
            let db_arc = user_db.0.clone();
            let staging_path = db_staging
                .to_str()
                .ok_or_else(|| crate::server_error::ServerError::from("invalid staging path"))?
                .to_string();
            tokio::task::spawn_blocking(move || db_arc.blocking_read().backup(&staging_path))
                .await??;
        } else {
            let db_arc = user_db.0.clone();
            let staging_path = db_staging
                .to_str()
                .ok_or_else(|| crate::server_error::ServerError::from("invalid staging path"))?
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
                    .map_err(|e| crate::server_error::ServerError::from(e.to_string()))?;
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
        .ok_or_else(|| crate::server_error::ServerError::from("invalid staging path"))?
        .to_string();
    let log_inner = cluster_log_ref.0.clone();
    tokio::task::spawn_blocking(move || log_inner.blocking_read().backup(&cluster_log_staging_str))
        .await??;

    let server_db_tmp = staging_dir.join(".server_db_tmp");
    let server_db_tmp_str = server_db_tmp
        .to_str()
        .ok_or_else(|| crate::server_error::ServerError::from("invalid staging path"))?
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

fn snapshot_body_stream(
    staging_dir: PathBuf,
    start_offset: u64,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
    in_flight: Arc<AtomicUsize>,
) -> ServerResult<Body> {
    let files = list_snapshot_files(&staging_dir)?;
    Ok(Body::from_stream(snapshot_file_stream(
        files,
        start_offset,
        log_index,
        log_term,
        log_commit,
        in_flight,
    )))
}

fn snapshot_file_stream(
    files: Vec<(String, PathBuf)>,
    start_offset: u64,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
    in_flight: Arc<AtomicUsize>,
) -> impl futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send + 'static {
    let file_count = files.len() as u64;

    async_stream::try_stream! {
        struct Guard(Arc<AtomicUsize>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::Release);
            }
        }
        let _guard = Guard(in_flight);

        let mut pos = 0u64;

        let mut header = [0u8; 32];
        header[..8].copy_from_slice(&log_index.to_le_bytes());
        header[8..16].copy_from_slice(&log_term.to_le_bytes());
        header[16..24].copy_from_slice(&log_commit.to_le_bytes());
        header[24..].copy_from_slice(&file_count.to_le_bytes());

        if 32u64 > start_offset {
            let skip = start_offset.saturating_sub(pos) as usize;
            yield bytes::Bytes::copy_from_slice(&header[skip..]);
        }
        pos = 32;

        let mut buf = vec![0u8; SNAPSHOT_STREAM_CHUNK];

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
                yield bytes::Bytes::copy_from_slice(&meta[skip..]);
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
                    yield bytes::Bytes::copy_from_slice(&buf[..n]);
                }
            }
            pos = data_end;
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
                .map_err(|e| crate::server_error::ServerError::from(e.to_string()))?
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
