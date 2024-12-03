use crate::cluster::Cluster;
use crate::config::Config;
use crate::error_code::ErrorCode;
use crate::raft::Request;
use crate::raft::Response;
use crate::raft::ResponseType;
use crate::server_error::ServerResult;
use crate::user_id::ClusterId;
use agdb_api::ClusterStatus;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

pub(crate) async fn cluster(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    request: Json<Request>,
) -> ServerResult<(StatusCode, Json<Response>)> {
    if let Some(raft) = &cluster.raft {
        let response = raft.write().await.request(&request).await;
        Ok((StatusCode::OK, Json(response)))
    } else {
        Ok((
            ErrorCode::ClusterUninitialized.into(),
            Json(Response {
                target: request.index,
                result: ResponseType::Ok,
            }),
        ))
    }
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
    let leader = if let Some(raft) = cluster.raft.as_ref() {
        raft.read().await.leader()
    } else {
        None
    };

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
