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

        for node in &config.cluster {
            let status = if node == &config.address {
                true
            } else {
                let url = format!("{}api/v1/status", node.as_str());
                tracing::info!("URL: {}", url);
                let response = client
                    .get(&url)
                    .timeout(std::time::Duration::from_secs(1))
                    .send()
                    .await;
                response.is_ok() && response?.status().is_success()
            };

            statuses.push(ClusterStatus {
                address: node.as_str().to_string(),
                status,
                leader: false,
                term: 0,
                commit: 0,
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
