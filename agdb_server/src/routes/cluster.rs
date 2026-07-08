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
use agdb_api::UserLogin;
use axum::Json;
use axum::body::Body;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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

pub(crate) async fn snapshot(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
) -> ServerResult<axum::response::Response> {
    cluster.snapshot_in_flight.store(true, Ordering::Relaxed);

    let result = build_snapshot(&cluster, &config).await;

    cluster.snapshot_in_flight.store(false, Ordering::Relaxed);

    result
}

async fn build_snapshot(
    cluster: &Cluster,
    config: &Config,
) -> ServerResult<axum::response::Response> {
    let data_dir = std::path::Path::new(&config.data_dir);
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    let (log_index, log_term, log_commit) = {
        let raft = cluster.raft.read().await;
        let _snapshot_lock = raft.storage.snapshot_lock.clone().write_owned().await;

        let dbs = raft.storage.db.dbs().await?;

        let server_db_guard = raft.storage.db.db.read().await;
        let server_db_data = std::fs::read(data_dir.join(SERVER_DB_FILE))?;
        drop(server_db_guard);

        files.push((SERVER_DB_FILE.to_string(), server_db_data));

        let cluster_log_data = std::fs::read(data_dir.join(CLUSTER_LOG_FILE))?;
        let log_index = raft.storage.index;
        let log_term = raft.storage.term;
        let log_commit = raft.storage.commit;
        drop(raft);

        files.push((CLUSTER_LOG_FILE.to_string(), cluster_log_data));

        for db in dbs {
            collect_db_files(data_dir, &db.owner, &db.db, config, &mut files)?;
        }

        (log_index, log_term, log_commit)
    };

    let file_count = files.len() as u64;
    let mut buf: Vec<u8> = Vec::new();

    buf.extend_from_slice(&log_index.to_le_bytes());
    buf.extend_from_slice(&log_term.to_le_bytes());
    buf.extend_from_slice(&log_commit.to_le_bytes());
    buf.extend_from_slice(&file_count.to_le_bytes());

    for (path, data) in &files {
        let path_bytes = path.as_bytes();
        buf.extend_from_slice(&(path_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(path_bytes);
        buf.extend_from_slice(&(data.len() as u64).to_le_bytes());
        buf.extend_from_slice(data);
    }

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/octet-stream")
        .body(Body::from(buf))
        .map_err(|e| crate::server_error::ServerError::from(e.to_string()))
}

fn collect_db_files(
    data_dir: &std::path::Path,
    owner: &str,
    db: &str,
    config: &Config,
    files: &mut Vec<(String, Vec<u8>)>,
) -> ServerResult<()> {
    let candidates = [
        db_pool::db_file(owner, db, config),
        db_pool::db_audit_file(owner, db, config),
        db_pool::db_backup_file(owner, db, config),
        db_pool::db_backup_audit_file(owner, db, config),
    ];

    for path in &candidates {
        if path.exists() {
            let relative = path.strip_prefix(data_dir).map_err(|_| {
                crate::server_error::ServerError::from(format!(
                    "Snapshot file is outside data_dir: {}",
                    path.to_string_lossy()
                ))
            })?;
            let relative = relative.to_string_lossy().to_string();
            let data = std::fs::read(path)?;
            files.push((relative, data));
        }
    }

    Ok(())
}
