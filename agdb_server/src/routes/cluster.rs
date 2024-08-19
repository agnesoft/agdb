use crate::config::Config;
use crate::server_error::ServerResult;
use agdb_api::ClusterStatus;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

#[utoipa::path(get,
    path = "/api/v1/cluster/status",
    tag = "agdb",
    responses(
         (status = 200, description = "Cluster status", body = Vec<ClusterStatus>),
    )
)]
pub(crate) async fn status(
    State(config): State<Config>,
) -> ServerResult<(StatusCode, Json<Vec<ClusterStatus>>)> {
    let mut statuses = Vec::with_capacity(config.cluster.len());
    let client = reqwest::Client::new();

    for node in &config.cluster {
        let status = if node == &config.address {
            true
        } else {
            let url = format!("{}api/v1/status", node.as_str());
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

    Ok((StatusCode::OK, Json(statuses)))
}
