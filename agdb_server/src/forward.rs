use crate::raft::Storage;
use crate::server_state::ServerState;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use reqwest::StatusCode;

const REDIRECT_PATHS: [&str; 15] = [
    "/add",
    "/backup",
    "/change_password",
    "/clear",
    "/cluster/login",
    "/cluster/logout",
    "/cluster/admin/logout",
    "/convert",
    "/copy",
    "/delete",
    "/exec",
    "/optimize",
    "/remove",
    "/rename",
    "/restore",
];

pub(crate) async fn forward_to_leader(
    state: State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    if REDIRECT_PATHS
        .iter()
        .any(|pattern| request.uri().path().ends_with(pattern))
    {
        let leader = state.cluster.raft.read().await.leader();
        if let Some(leader) = leader {
            if state.cluster.index != leader as usize {
                let mut response = match state.cluster.nodes[leader as usize]
                    .forward(request, state.cluster.index)
                    .await
                {
                    Ok(r) => r,
                    Err(r) => r,
                };

                if response.status().is_success() {
                    if let Some(commit_index) = response.headers_mut().remove("commit-index") {
                        if let Ok(commit_index) = commit_index.to_str() {
                            if let Ok(commit_index) = commit_index.parse::<u64>() {
                                while state.cluster.raft.read().await.storage.log_executed()
                                    < commit_index
                                {
                                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                                }
                            }
                        }
                    }
                }

                return response;
            }
        } else {
            return Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body("Cluster is not ready yet".into())
                .expect("cluster not ready yet response");
        }
    }

    next.run(request).await
}
