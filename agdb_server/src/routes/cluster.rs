use crate::cluster::Cluster;
use crate::cluster::ClusterState;
use crate::config::Config;
use crate::error_code::ErrorCode;
use crate::server_error::ServerResult;
use crate::user_id::ClusterId;
use agdb_api::ClusterStatus;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::sync::atomic::Ordering;
use std::time::Instant;

#[derive(Deserialize)]
pub(crate) struct ClusterParams {
    cluster_hash: u64,
    term: u64,
    leader: usize,
}

pub(crate) async fn heartbeat(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    request: Query<ClusterParams>,
) -> ServerResult<(StatusCode, Json<String>)> {
    if cluster.cluster_hash != request.cluster_hash {
        return Ok((
            ErrorCode::ClusterHashMismatch.into(),
            Json(format!(
                "Cluster hash mismatch: expected {}, got {}",
                cluster.cluster_hash, request.cluster_hash
            )),
        ));
    }

    let current_term = cluster.data.read().await.term;

    if request.term < current_term {
        return Ok((
            ErrorCode::TermMismatch.into(),
            Json(format!(
                "Term mismatch: expected higher term than {}, got {}",
                current_term, request.term
            )),
        ));
    }

    let state = cluster.data.read().await.state;

    if let ClusterState::Follower(leader) = state {
        if leader == request.leader && request.term == current_term {
            return Ok((StatusCode::OK, Json(String::new())));
        }
    }

    tracing::info!(
        "[{}] Becoming a follower of node {}, term: {}",
        cluster.index,
        request.leader,
        request.term
    );

    let mut data = cluster.data.write().await;
    data.term = request.term;
    data.state = ClusterState::Follower(request.leader);
    data.leader.store(false, Ordering::Relaxed);
    data.timer = Instant::now();

    Ok((StatusCode::OK, Json(String::new())))
}

pub(crate) async fn vote(
    _cluster_id: ClusterId,
    State(cluster): State<Cluster>,
    request: Query<ClusterParams>,
) -> ServerResult<(StatusCode, Json<String>)> {
    if cluster.cluster_hash != request.cluster_hash {
        return Ok((
            ErrorCode::ClusterHashMismatch.into(),
            Json(format!(
                "Cluster hash mismatch: expected local ({}) == other ({})",
                cluster.cluster_hash, request.cluster_hash
            )),
        ));
    }

    let current_leader = cluster.leader().await;

    if let Some(leader) = current_leader {
        return Ok((
            ErrorCode::LeaderExists.into(),
            Json(format!("Leader already exists: node {}", leader)),
        ));
    }

    let current_term = cluster.data.read().await.term;

    if request.term <= current_term {
        return Ok((
            ErrorCode::TermMismatch.into(),
            Json(format!(
                "Term mismatch: epxected current ({}) < requested ({})",
                current_term, request.term
            )),
        ));
    }

    let voted = cluster.data.read().await.voted;

    if request.term <= voted {
        return Ok((
            ErrorCode::AlreadyVoted.into(),
            Json(format!(
                "Already voted: expected last vote ({voted}) < {}",
                request.term
            )),
        ));
    }

    let mut data = cluster.data.write().await;
    data.term = request.term;
    data.voted = request.term;
    data.timer = Instant::now();

    Ok((StatusCode::OK, Json(String::new())))
}

#[utoipa::path(get,
    path = "/api/v1/cluster/status",
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

    let leader;
    let term;

    {
        let data = cluster.data.read().await;
        leader = cluster.leader().await;
        term = data.term;
    }

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
                        leader: status && Some(index) == leader,
                        term,
                        commit: 0,
                    },
                )
            }));
        } else {
            let status = &mut statuses[index];
            status.address = node.as_str().to_string();
            status.status = true;
            status.leader = Some(index) == leader;
            status.term = term;
            status.commit = 0;
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
