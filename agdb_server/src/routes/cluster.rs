use crate::action::cluster_login::ClusterLogin;
use crate::action::ClusterAction;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::raft::Request;
use crate::raft::Response;
use crate::routes::user::do_login;
use crate::server_db::ServerDb;
use crate::server_error::ServerResponse;
use crate::server_error::ServerResult;
use crate::user_id::AdminId;
use crate::user_id::ClusterId;
use crate::user_id::UserId;
use agdb_api::ClusterStatus;
use agdb_api::UserLogin;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

pub(crate) async fn cluster(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    request: Json<Request<ClusterAction>>,
) -> ServerResult<(StatusCode, Json<Response>)> {
    let response = cluster.raft.write().await.request(&request).await;
    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(post,
    path = "/api/v1/admin/cluster/{username}/logout",
    operation_id = "admin_cluster_logout",
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

    let commit_index = cluster
        .append(ClusterLogin {
            user: username,
            new_token: String::new(),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        [("commit-index", commit_index.to_string())],
    ))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/login",
    operation_id = "cluster_login",
    tag = "agdb",
    request_body = UserLogin,
    responses(
         (status = 200, description = "login successful", body = String),
         (status = 401, description = "invalid credentials"),
    )
)]
pub(crate) async fn login(
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
    Json(request): Json<UserLogin>,
) -> ServerResponse<impl IntoResponse> {
    let (token, user_id) = do_login(&server_db, &request.username, &request.password).await?;
    let mut commit_index = 0;

    if user_id.is_some() {
        commit_index = cluster
            .append(ClusterLogin {
                user: request.username,
                new_token: token.clone(),
            })
            .await?;
    }

    Ok((
        StatusCode::OK,
        [("commit-index", commit_index.to_string())],
        Json(token),
    ))
}

#[utoipa::path(post,
    path = "/api/v1/cluster/logout",
    operation_id = "cluster_logout",
    tag = "agdb",
    security(("Token" = [])),
    responses(
         (status = 201, description = "user logged out"),
         (status = 401, description = "invalid credentials")
    )
)]
pub(crate) async fn logout(
    user: UserId,
    State(server_db): State<ServerDb>,
    State(cluster): State<Cluster>,
) -> ServerResponse<impl IntoResponse> {
    let token = server_db.user_token(user.0).await?;
    let mut commit_index = 0;

    if !token.is_empty() {
        commit_index = cluster
            .append(ClusterLogin {
                user: server_db.user_name(user.0).await?,
                new_token: String::new(),
            })
            .await?;
    }

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
    let mut statuses = vec![ClusterStatus::default(); config.cluster.len()];
    let mut tasks = Vec::new();
    let leader = cluster.raft.read().await.leader();

    for (index, node) in config.cluster.iter().enumerate() {
        if index != cluster.index {
            let address = node.as_str().to_string();
            let url = format!("{}api/v1/status", node.as_str());

            tasks.push(tokio::spawn(async move {
                let client = reqwest::Client::new();

                let response = client
                    .get(&url)
                    .timeout(std::time::Duration::from_secs(1))
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
