use crate::action::ClusterAction;
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
use crate::routes::user::LogoutAllQuery;
use crate::routes::user::do_login;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::server_error::ServerResult;
use crate::user_id::AdminId;
use crate::user_id::ClusterId;
use crate::user_id::UserAgent;
use crate::user_id::UserId;
use crate::user_id::UserIdToken;
use agdb_api::ClusterStatus;
use agdb_api::UserLogin;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub(crate) async fn cluster(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    request: Json<Request<ClusterAction>>,
) -> ServerResult<(StatusCode, Json<Response>)> {
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
    ),
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "admin only"),
         (status = 404, description = "user not found"),
    )
)]
pub(crate) async fn admin_logout(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path(username): Path<String>,
) -> ServerResponse<impl IntoResponse> {
    let _user_id = server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster.exec(RemoveUserTokens { user: username }).await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(delete,
    path = "/api/v1/cluster/admin/user/{username}/logout/{session}",
    operation_id = "cluster_admin_user_logout_session",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("username" = String, Path, description = "user name"),
        ("session" = i64, Path, description = "session id"),
    ),
    responses(
         (status = 201, description = "session revoked"),
         (status = 401, description = "admin only"),
         (status = 404, description = "user or session not found"),
    )
)]
pub(crate) async fn admin_logout_session(
    _admin: AdminId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path((username, session)): Path<(String, i64)>,
) -> ServerResponse<impl IntoResponse> {
    let _user_id = server_db.user_id(&username).await?;

    let (commit_index, _result) = cluster
        .exec(RemoveUserSession {
            user: username,
            session,
        })
        .await?;

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
    let (_user_id, token) = do_login(&server_db, &request.username, &request.password).await?;
    let (commit_index, _result) = cluster
        .exec(SaveUserToken {
            user: request.username,
            new_token: token.clone(),
            agent: agent.0,
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
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "invalid credentials")
    )
)]
pub(crate) async fn logout(
    user: UserIdToken,
    State(cluster): State<Cluster>,
) -> ServerResponse<impl IntoResponse> {
    let (commit_index, _result) = cluster
        .exec(RemoveUserToken {
            token: user.0.clone(),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/user/logout_all",
    operation_id = "cluster_user_logout_all",
    tag = "agdb",
    security(("Token" = [])),
    params(
        LogoutAllQuery,
    ),
    responses(
         (status = 201, description = "user logged out from all sessions"),
         (status = 401, description = "invalid credentials")
    )
)]
pub(crate) async fn user_logout_all(
    user: UserId,
    token: UserIdToken,
    Query(request): Query<LogoutAllQuery>,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
) -> ServerResponse<impl IntoResponse> {
    let username = server_db.user_name(user.0).await?;
    let (commit_index, _result) = if request.include_self {
        cluster.exec(RemoveUserTokens { user: username }).await?
    } else {
        cluster
            .exec(RemoveUserTokensExcept {
                user: username,
                token: token.0,
            })
            .await?
    };

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(delete,
    path = "/api/v1/cluster/user/logout/{session}",
    operation_id = "cluster_user_logout_session",
    tag = "agdb",
    security(("Token" = [])),
    params(
        ("session" = i64, Path, description = "session id"),
    ),
    responses(
         (status = 201, description = "session revoked"),
         (status = 401, description = "invalid credentials"),
         (status = 404, description = "session not found"),
    )
)]
pub(crate) async fn user_logout_session(
    user: UserId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Path(session): Path<i64>,
) -> ServerResponse<impl IntoResponse> {
    let username = server_db.user_name(user.0).await?;
    let (commit_index, _result) = cluster
        .exec(RemoveUserSession {
            user: username,
            session,
        })
        .await?;

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
