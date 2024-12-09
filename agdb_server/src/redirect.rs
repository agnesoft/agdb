use crate::server_state::ServerState;
use axum::body::Body;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use reqwest::StatusCode;

const REDIRECT_PATHS: [&str; 14] = [
    "/add",
    "/backup",
    "/change_password",
    "/clear",
    "/convert",
    "/copy",
    "/delete",
    "/exec",
    "/login",
    "/logout",
    "/optimize",
    "/remove",
    "/rename",
    "/restore",
];

pub(crate) async fn cluster_redirect(
    state: State<ServerState>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    if REDIRECT_PATHS
        .iter()
        .any(|pattern| request.uri().path().ends_with(pattern))
    {
        if let Some(leader) = state.cluster.raft.read().await.leader() {
            if state.cluster.index != leader as usize {
                return Ok(Response::builder()
                    .status(StatusCode::PERMANENT_REDIRECT)
                    .header("Location", state.config.cluster[leader as usize].as_str())
                    .body(Body::empty())
                    .expect("pemanent redirect response"));
            }
        } else {
            return Ok(Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body("Cluster is not ready yet".into())
                .expect("cluster not ready yet response"));
        }
    }

    Ok(next.run(request).await)
}
