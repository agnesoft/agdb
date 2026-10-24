use crate::action::ClusterAction;
use crate::action::ResyncTarget;
use crate::action::remove_all_tokens::RemoveAllTokens;
use crate::action::remove_user_session::RemoveUserSession;
use crate::action::remove_user_token::RemoveUserToken;
use crate::action::remove_user_tokens::RemoveUserTokens;
use crate::action::remove_user_tokens_except::RemoveUserTokensExcept;
use crate::action::save_user_token::SaveUserToken;
use crate::cluster;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::raft::Request;
use crate::raft::Response;
use crate::raft::ResponseType;
use crate::resync::RESYNC_SERVER_DB_NAME;
use crate::resync::RESYNC_SERVER_DB_OWNER;
use crate::routes::user::LOGOUT_ALL_SESSIONS;
use crate::routes::user::LOGOUT_OTHER_SESSIONS;
use crate::routes::user::LogoutQuery;
use crate::routes::user::do_login;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResponse;
use crate::server_error::ServerResult;
use crate::user_id::AdminId;
use crate::user_id::ClusterId;
use crate::user_id::UserAgent;
use crate::user_id::UserName;
use crate::user_id::UserToken;
use crate::wire;
use agdb_api::ClusterStatus;
use agdb_api::UserLogin;
use axum::Json;
use axum::body::Bytes;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use std::time::Duration;

struct ResyncInProgressGuard {
    set: Arc<Mutex<HashSet<(String, String)>>>,
    key: (String, String),
}

impl Drop for ResyncInProgressGuard {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.set.lock() {
            guard.remove(&self.key);
        }
    }
}

// Accepts the binary protocol and JSON (older leaders) and answers in kind.
pub(crate) async fn cluster(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    State(config): State<Config>,
    headers: HeaderMap,
    body: Bytes,
) -> ServerResult<axum::response::Response> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let binary = content_type == wire::CONTENT_TYPE;

    let request: Request<ClusterAction> = if binary {
        wire::decode(&body)
            .map_err(|e| ServerError::new(StatusCode::BAD_REQUEST, &e.description))?
    } else if content_type.starts_with("application/json") {
        serde_json::from_slice(&body)
            .map_err(|e| ServerError::new(StatusCode::BAD_REQUEST, &e.to_string()))?
    } else {
        return Ok(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response());
    };

    if cluster.resync.load(Ordering::Relaxed) {
        let response = Response::new(request.index, ResponseType::Resyncing);
        return Ok(reply(with_node_state(response, &cluster), binary));
    }

    let response = with_node_state(cluster.raft.write().await.request(&request).await, &cluster);

    if let Some(dbs) = request.resync_dbs()
        && response.result == ResponseType::Ok
    {
        // Drain matching failed indices from memory synchronously so
        // the very next OkReport won't re-trigger ResyncDbs.
        {
            let raft = cluster.raft.read().await;
            let all: Vec<u64> = raft
                .storage
                .failed_indices
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            let mut matched = Vec::new();
            for &idx in &all {
                if let Ok(Some(action)) = raft.storage.cluster_log.action_at_index(idx).await
                    && action.resync_targets().iter().any(|t| match t {
                        ResyncTarget::UserDb(o, d) => {
                            dbs.iter().any(|(ow, db)| *o == *ow && *d == *db)
                        }
                        ResyncTarget::ServerDb => dbs.iter().any(|(ow, db)| {
                            ow == RESYNC_SERVER_DB_OWNER && db == RESYNC_SERVER_DB_NAME
                        }),
                    })
                {
                    matched.push(idx);
                }
            }
            if !matched.is_empty() {
                raft.storage
                    .failed_indices
                    .write()
                    .unwrap_or_else(|e| e.into_inner())
                    .retain(|idx| !matched.contains(idx));
            }
        }

        let resync_dbs = dbs.clone();
        let resync_cluster = cluster.clone();
        let resync_config = config.clone();

        tokio::spawn(async move {
            for (owner, db) in resync_dbs {
                let key = (owner.clone(), db.clone());

                // Skip if a resync is already in flight for this (owner, db).
                {
                    let mut in_progress = resync_cluster
                        .resync_in_progress
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());

                    if !in_progress.insert(key.clone()) {
                        crate::info!(
                            "[{}] Skipping per-DB resync for {}/{}: already in progress",
                            resync_cluster.index,
                            owner,
                            db,
                        );
                        continue;
                    }
                }

                // RAII guard: removes the key on drop (normal exit or panic).
                let _guard = ResyncInProgressGuard {
                    set: resync_cluster.resync_in_progress.clone(),
                    key: key.clone(),
                };

                crate::info!(
                    "[{}] Leader-instructed per-DB resync: {}/{}",
                    resync_cluster.index,
                    owner,
                    db
                );
                if let Err(e) =
                    crate::resync::resync_single_db(&resync_cluster, &resync_config, &owner, &db)
                        .await
                {
                    crate::error!(
                        "[{}] Per-DB resync failed for {}/{}: {:?}",
                        resync_cluster.index,
                        owner,
                        db,
                        e
                    );
                    // Rebuild failed indices from persistent log so the
                    // leader retries on the next heartbeat.
                    let raft = resync_cluster.raft.read().await;
                    if let Ok(indices) = raft.storage.cluster_log.failed_indices().await {
                        *raft
                            .storage
                            .failed_indices
                            .write()
                            .unwrap_or_else(|e| e.into_inner()) = indices;
                    }
                } else {
                    resync_cluster
                        .raft
                        .read()
                        .await
                        .storage
                        .clear_resolved_indices(&owner, &db)
                        .await;
                }
            }
        });
    }

    Ok(reply(response, binary))
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

fn reply(response: Response, binary: bool) -> axum::response::Response {
    if binary {
        (
            [(header::CONTENT_TYPE, wire::CONTENT_TYPE)],
            wire::encode(&response),
        )
            .into_response()
    } else {
        Json(response).into_response()
    }
}

fn with_node_state(mut response: Response, cluster: &Cluster) -> Response {
    response.executed_index = *cluster.executed.borrow();
    response
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
    headers: HeaderMap,
) -> ServerResult<(StatusCode, Json<Vec<ClusterStatus>>)> {
    if config.cluster.is_empty() {
        let status = ClusterStatus {
            address: config.server_url(),
            status: true,
            leader: true,
            executed_index: *cluster.executed.borrow(),
            resync: false,
        };
        return Ok((StatusCode::OK, Json(vec![status])));
    }

    let leader = cluster.raft.read().await.leader();

    let mut statuses = match leader {
        Some(leader) if leader as usize == cluster.index => leader_view(&config, &cluster),
        // A forwarded request is answered locally even if leadership moved, to avoid ping-pong.
        Some(leader) if !headers.contains_key("forwarded-by") => {
            match leader_status(&cluster, leader as usize).await {
                Some(statuses) => statuses,
                None => probe_view(&config, &cluster, Some(leader)).await?,
            }
        }
        _ => probe_view(&config, &cluster, leader).await?,
    };

    statuses.sort_by(|a, b| a.address.cmp(&b.address));

    Ok((StatusCode::OK, Json(statuses)))
}

fn leader_view(config: &Config, cluster: &Cluster) -> Vec<ClusterStatus> {
    // Wider than the term timeout so a peer busy with a slow append is not reported down.
    let alive = Duration::from_millis(config.cluster_term_timeout_ms * 2);

    config
        .cluster
        .iter()
        .zip(&cluster.nodes)
        .enumerate()
        .map(|(index, (address, node))| {
            if index == cluster.index {
                ClusterStatus {
                    address: address.as_str().to_string(),
                    status: true,
                    leader: true,
                    executed_index: *cluster.executed.borrow(),
                    resync: false,
                }
            } else {
                let peer = node.peer_state();
                ClusterStatus {
                    address: address.as_str().to_string(),
                    status: peer.last_response.is_some_and(|t| t.elapsed() < alive),
                    leader: false,
                    executed_index: peer.executed_index,
                    resync: peer.resync,
                }
            }
        })
        .collect()
}

async fn leader_status(cluster: &Cluster, leader: usize) -> Option<Vec<ClusterStatus>> {
    let node = &cluster.nodes[leader];
    let response = node
        .http_client()
        .get(format!("{}/api/v1/cluster/status", node.base_url()))
        .header("forwarded-by", cluster.index)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    response.json().await.ok()
}

async fn probe_view(
    config: &Config,
    cluster: &Cluster,
    leader: Option<u64>,
) -> ServerResult<Vec<ClusterStatus>> {
    let mut statuses = vec![ClusterStatus::default(); config.cluster.len()];
    let mut tasks = Vec::new();

    for (index, node) in config.cluster.iter().enumerate() {
        if index != cluster.index {
            let address = node.as_str().to_string();
            let url = format!("{}/api/v1/status", node.trim_end_matches("/"));
            let client = cluster::reqwest_client(config)?;

            tasks.push(tokio::spawn(async move {
                let response = client
                    .get(&url)
                    .timeout(Duration::from_secs(5))
                    .send()
                    .await;
                let status = response.is_ok_and(|r| r.status().is_success());

                (
                    index,
                    ClusterStatus {
                        address,
                        status,
                        leader: status && Some(index as u64) == leader,
                        ..Default::default()
                    },
                )
            }));
        } else {
            statuses[index] = ClusterStatus {
                address: node.as_str().to_string(),
                status: true,
                leader: Some(index as u64) == leader,
                executed_index: *cluster.executed.borrow(),
                resync: cluster.resync.load(Ordering::Relaxed),
            };
        }
    }

    for task in tasks {
        if let Ok((index, status)) = task.await {
            statuses[index] = status;
        }
    }

    Ok(statuses)
}
