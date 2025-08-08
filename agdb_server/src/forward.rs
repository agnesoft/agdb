use crate::server_state::ServerState;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use reqwest::StatusCode;

const REDIRECT_PATHS: [&str; 14] = [
    "/add",
    "/backup",
    "/change_password",
    "/clear",
    "/cluster/user/login",
    "/cluster/user/logout",
    "/convert",
    "/copy",
    "/delete",
    "/exec_mut",
    "/optimize",
    "/remove",
    "/rename",
    "/restore",
];

fn is_redirect_path(request: &Request) -> bool {
    let path = request.uri().path();

    REDIRECT_PATHS.iter().any(|pattern| path.ends_with(pattern))
        || (path.ends_with("/logout") && path.contains("/cluster/admin/user/"))
}

pub(crate) async fn forward_to_leader(
    state: State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    if is_redirect_path(&request) {
        let leader = state.cluster.raft.read().await.leader();
        if let Some(leader) = leader {
            if state.cluster.index != leader as usize {
                let mut notifier = state.cluster.raft.read().await.storage.subscribe().await;

                let mut response = match state.cluster.nodes[leader as usize]
                    .forward(request, state.cluster.index)
                    .await
                {
                    Ok(r) => r,
                    Err(r) => r,
                };

                if response.status().is_success()
                    && let Some(commit_index) = response.headers_mut().remove("commit-index")
                    && let Ok(commit_index) = commit_index.to_str()
                    && let Ok(commit_index) = commit_index.parse::<u64>()
                {
                    while let Ok(value) = notifier.recv().await {
                        if value == commit_index {
                            break;
                        }
                    }
                }

                return response;
            }
        } else {
            return Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body("cluster is not ready yet".into())
                .expect("cluster not ready yet response");
        }
    }

    next.run(request).await
}
