pub(crate) mod admin;
pub(crate) mod db;
pub(crate) mod user;

use crate::config::Config;
use crate::server_error::ServerResult;
use agdb_api::ClusterStatus;
use agdb_api::StatusParams;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(get,
    path = "/api/v1/status",
    params(
        ("cluster" = bool, description = "get cluster status"),
    ),
    responses(
         (status = 200, description = "Server is ready", body = Vec<ClusterStatus>),
    )
)]
pub(crate) async fn status(
    State(config): State<Config>,
    Query(status_params): Query<StatusParams>,
) -> ServerResult<(StatusCode, Json<Vec<ClusterStatus>>)> {
    let statuses = if status_params.cluster.unwrap_or_default() {
        let mut statuses = Vec::with_capacity(config.cluster.len());
        let client = reqwest::Client::new();
        let local_node = format!("{}:{}", config.host, config.port);

        for node in &config.cluster {
            let status = if node == &local_node {
                true
            } else {
                let url = format!("http://{node}/api/v1/status");
                let response = client
                    .get(&url)
                    .timeout(std::time::Duration::from_secs(1))
                    .send()
                    .await;
                response.is_ok() && response?.status().is_success()
            };

            statuses.push(ClusterStatus {
                address: node.clone(),
                status,
            });
        }

        statuses
    } else {
        vec![]
    };

    Ok((StatusCode::OK, Json(statuses)))
}

pub(crate) async fn test_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
