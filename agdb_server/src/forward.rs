use crate::server_state::ServerState;
use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
use reqwest::StatusCode;
use std::sync::atomic::Ordering;
use std::time::Duration;

const EXECUTED_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

const REDIRECT_PATHS: [&str; 16] = [
    "/add",
    "/backup",
    "/change_password",
    "/clear",
    "/cluster/user/login",
    "/cluster/user/logout",
    "/cluster/admin/user/logout_all",
    "/convert",
    "/copy",
    "/delete",
    "/exec_mut",
    "/optimize",
    "/remove",
    "/rename",
    "/restore",
    "/rollback",
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
    let body_limit = state.config.request_body_limit as usize;

    let path = request.uri().path();
    let served_locally = path.ends_with("/cluster") || path.ends_with("/cluster/snapshot");

    if !served_locally && state.cluster.resync.load(Ordering::Relaxed) {
        // Not inlined into the condition: an `if let` temporary would keep the
        // raft guard alive across the forward and stall every other request.
        let leader = state.cluster.raft.read().await.leader();

        if let Some(leader) = leader
            && state.cluster.index != leader as usize
        {
            return state.cluster.nodes[leader as usize]
                .forward(request, state.cluster.index, body_limit)
                .await
                .unwrap_or_else(|r| r);
        }
    }

    if is_redirect_path(&request) {
        let leader = state.cluster.raft.read().await.leader();
        if let Some(leader) = leader {
            if state.cluster.index != leader as usize {
                let mut response = match state.cluster.nodes[leader as usize]
                    .forward(request, state.cluster.index, body_limit)
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
                    let mut executed = state.cluster.executed.clone();

                    // The write is committed either way; on timeout only read-your-writes on this node is lost.
                    match tokio::time::timeout(
                        EXECUTED_WAIT_TIMEOUT,
                        executed.wait_for(|index| *index >= commit_index),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {}
                        Ok(Err(_)) => crate::warn!(
                            "[{}] Executed index channel closed while waiting for {commit_index}",
                            state.cluster.index
                        ),
                        Err(_) => crate::warn!(
                            "[{}] Timed out waiting for local execution of {commit_index} (executed: {})",
                            state.cluster.index,
                            *state.cluster.executed.borrow()
                        ),
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
