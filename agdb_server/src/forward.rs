use crate::server_state::ServerState;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use reqwest::StatusCode;

const REDIRECT_PATHS: [&str; 13] = [
    "/add",
    "/backup",
    "/change_password",
    "/clear",
    "/cluster/login",
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
                return match state.cluster.nodes[leader as usize]
                    .forward(request, state.cluster.index)
                    .await
                {
                    Ok(r) => r,
                    Err(r) => r,
                };
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
