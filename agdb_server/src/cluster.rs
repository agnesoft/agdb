use crate::action::Action;
use crate::action::ClusterAction;
use crate::action::ClusterActionResult;
use crate::action::ResyncTarget;
use crate::cluster_log::CLUSTER_LOG_FILE;
use crate::cluster_log::ClusterLog;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::raft;
use crate::raft::Log;
use crate::raft::Request;
use crate::raft::Response;
use crate::raft::Storage;
use crate::resync::RESYNC_SERVER_DB_NAME;
use crate::resync::RESYNC_SERVER_DB_OWNER;
use crate::server_db::SERVER_DB_FILE;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use agdb::DbId;
use agdb::StableHash;
use agdb_api::HttpClient;
use agdb_api::ReqwestClient;
use axum::body::Body;
use axum::extract::Request as AxumRequest;
use axum::http::HeaderMap;
use axum::response::Response as AxumResponse;
use futures::FutureExt;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock as StdRwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::signal;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;
use tokio::sync::watch;
use tokio::time::Instant;

pub(crate) type Cluster = Arc<ClusterImpl>;

type ClusterNode = Arc<ClusterNodeImpl>;
type ResultNotifier = Sender<ServerResult<(u64, ClusterActionResult)>>;
type ClusterResponseReceiver = UnboundedReceiver<(Request<ClusterAction>, Response)>;

const FORWARD_CONNECT_ATTEMPTS: u32 = 3;
const FORWARD_CONNECT_BACKOFF: Duration = Duration::from_millis(200);

pub(crate) struct ClusterNodeImpl {
    client: ReqwestClient,
    url: String,
    base_url: String,
    base_path: String,
    token: Option<String>,
    requests_sender: UnboundedSender<Request<ClusterAction>>,
    requests_receiver: RwLock<UnboundedReceiver<Request<ClusterAction>>>,
    responses: UnboundedSender<(Request<ClusterAction>, Response)>,
    peer: Mutex<PeerState>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PeerState {
    pub(crate) last_response: Option<Instant>,
    pub(crate) executed_index: u64,
    pub(crate) resync: bool,
}

pub(crate) struct ClusterImpl {
    pub(crate) index: usize,
    pub(crate) nodes: Vec<ClusterNode>,
    pub(crate) raft: Arc<RwLock<raft::Cluster<ClusterAction, ResultNotifier, ClusterStorage>>>,
    pub(crate) responses: Option<RwLock<ClusterResponseReceiver>>,
    pub(crate) resync: Arc<AtomicBool>,
    pub(crate) snapshot_in_flight: Arc<AtomicUsize>,
    pub(crate) poisoned: Arc<AtomicBool>,
    pub(crate) resync_in_progress: Arc<Mutex<std::collections::HashSet<(String, String)>>>,
    pub(crate) executed: watch::Receiver<u64>,
}

impl ClusterImpl {
    pub(crate) async fn exec<T: Action + Into<ClusterAction>>(
        &self,
        action: T,
    ) -> ServerResult<(u64, ClusterActionResult)> {
        let (sender, receiver) =
            tokio::sync::oneshot::channel::<ServerResult<(u64, ClusterActionResult)>>();
        let requests = self
            .raft
            .write()
            .await
            .append(action.into(), Some(sender))
            .await?;

        for request in requests {
            self.nodes[request.target as usize]
                .requests_sender
                .send(request)?;
        }

        receiver.await?
    }
}

impl ClusterNodeImpl {
    fn new(
        address: &str,
        token: &str,
        responses: UnboundedSender<(Request<ClusterAction>, Response)>,
        config: &Config,
    ) -> ServerResult<Self> {
        let base = if address.starts_with("http") || address.starts_with("https") {
            address.to_string()
        } else {
            format!("http://{address}")
        };

        let (requests_sender, requests_receiver) = tokio::sync::mpsc::unbounded_channel();
        let base_url = base.trim_end_matches("/").to_string();

        Ok(Self {
            client: ReqwestClient::with_client(reqwest_client(config)?),
            url: format!("{base_url}/api/v1/cluster"),
            base_url,
            base_path: config.basepath.clone(),
            token: Some(token.to_string()),
            requests_sender,
            requests_receiver: RwLock::new(requests_receiver),
            responses,
            peer: Mutex::new(PeerState::default()),
        })
    }

    pub(crate) fn peer_state(&self) -> PeerState {
        *self.peer.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn record_peer_state(&self, response: &Response) {
        *self.peer.lock().unwrap_or_else(|e| e.into_inner()) = PeerState {
            last_response: Some(Instant::now()),
            executed_index: response.executed_index,
            resync: response.result == raft::ResponseType::Resyncing,
        };
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns a reference to the node's shared reqwest client,
    /// avoiding per-call TLS/connection-pool setup costs.
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        &self.client.client
    }

    fn error_response(status: StatusCode, message: &str) -> AxumResponse {
        AxumResponse::builder()
            .status(status)
            .body(message.to_owned().into())
            .expect("error response")
    }

    fn bad_request(message: &str) -> AxumResponse {
        Self::error_response(StatusCode::BAD_REQUEST, message)
    }

    /// Failures to reach the leader are reported as `503` when the request was never sent
    /// (safe to retry) and `502` when it may have been (outcome unknown).
    #[allow(clippy::result_large_err)]
    pub(crate) async fn forward(
        &self,
        axum_request: AxumRequest,
        local_index: usize,
        body_limit: usize,
    ) -> Result<AxumResponse, AxumResponse> {
        let (parts, body) = axum_request.into_parts();
        let path_query = parts.uri.path_and_query().ok_or(Self::bad_request(""))?;
        let path_str = path_query.as_str();
        let stripped_path = path_str.strip_prefix(&self.base_path).unwrap_or(path_str);
        let url = format!("{}{stripped_path}", self.base_url);
        let mut headers = HeaderMap::new();

        headers.insert("forwarded-by", local_index.into());

        if let Some(auth_header) = parts.headers.get("authorization") {
            headers.insert("authorization", auth_header.clone());
        }

        if let Some(content_type) = parts.headers.get("content-type") {
            headers.insert("content-type", content_type.clone());
        }

        if let Some(user_agent) = parts.headers.get("user-agent") {
            headers.insert("user-agent", user_agent.clone());
        }

        let method = reqwest::Method::from_str(parts.method.as_str())
            .map_err(|e| Self::bad_request(&e.to_string()))?;
        let body = axum::body::to_bytes(body, body_limit).await.map_err(|e| {
            let too_large =
                std::iter::successors(Some(&e as &dyn std::error::Error), |e| e.source())
                    .any(|e| e.is::<http_body_util::LengthLimitError>());
            let status = if too_large {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            };
            Self::error_response(status, &format!("failed to read request body: {e}"))
        })?;
        let mut attempt = 1;

        let mut response = loop {
            match self
                .client
                .client
                .request(method.clone(), &url)
                .headers(headers.clone())
                .body(body.clone())
                .send()
                .await
            {
                Ok(response) => break response,
                Err(e) if e.is_connect() && attempt < FORWARD_CONNECT_ATTEMPTS => {
                    crate::warn!(
                        "[{local_index}] Forward to leader failed to connect (attempt {attempt}/{FORWARD_CONNECT_ATTEMPTS}): {}",
                        error_chain(&e)
                    );
                    tokio::time::sleep(FORWARD_CONNECT_BACKOFF * attempt).await;
                    attempt += 1;
                }
                Err(e) if e.is_connect() => {
                    return Err(Self::error_response(
                        StatusCode::SERVICE_UNAVAILABLE,
                        &format!("forward to leader failed to connect: {}", error_chain(&e)),
                    ));
                }
                Err(e) => {
                    return Err(Self::error_response(
                        StatusCode::BAD_GATEWAY,
                        &format!("forward to leader failed: {}", error_chain(&e)),
                    ));
                }
            }
        };

        let mut axum_response = AxumResponse::builder().status(response.status());

        if let Some(headers) = axum_response.headers_mut() {
            std::mem::swap(headers, response.headers_mut())
        }

        axum_response
            .body(Body::from_stream(response.bytes_stream()))
            .map_err(|e| Self::error_response(StatusCode::BAD_GATEWAY, &e.to_string()))
    }

    async fn send(&self, request: &raft::Request<ClusterAction>) -> Option<raft::Response> {
        match self
            .client
            .post(&self.url, Some(request), &self.token)
            .await
        {
            Ok((_, response)) => Some(response),
            Err(e) => {
                crate::warn!(
                    "[{}] Error sending request to cluster node '{}': {:?}",
                    request.index,
                    request.target,
                    e
                );
                None
            }
        }
    }
}

pub(crate) async fn new(
    config: &Config,
    db: &ServerDb,
    cluster_log: &ClusterLog,
    db_pool: &DbPool,
) -> ServerResult<Cluster> {
    let index = config
        .cluster
        .iter()
        .position(|url| url == &config.server_url())
        .unwrap_or_default();
    let mut sorted_cluster: Vec<String> =
        config.cluster.iter().map(|url| url.to_string()).collect();
    sorted_cluster.sort();
    sorted_cluster.push(config.cluster_max_log_entries.to_string());
    let hash = sorted_cluster.stable_hash();
    let resync = Arc::new(AtomicBool::new(false));
    let snapshot_in_flight = Arc::new(AtomicUsize::new(0));
    let poisoned = Arc::new(AtomicBool::new(false));
    let storage = ClusterStorage::new(
        index,
        config.log_body_limit,
        db.clone(),
        cluster_log.clone(),
        db_pool.clone(),
        config.cluster_max_log_entries,
        snapshot_in_flight.clone(),
        poisoned.clone(),
    )
    .await?;
    let settings = raft::ClusterSettings {
        index: index as u64,
        hash,
        size: std::cmp::max(config.cluster.len() as u64, 1),
        election_factor_ms: config.cluster_election_factor_ms,
        heartbeat_timeout: Duration::from_millis(config.cluster_heartbeat_timeout_ms),
        term_timeout: Duration::from_millis(config.cluster_term_timeout_ms),
        max_log_entries: config.cluster_max_log_entries,
    };
    let executed = storage.executed.subscribe();
    let raft = Arc::new(RwLock::new(raft::Cluster::new(storage, settings)));
    let mut nodes = vec![];

    let responses = if !sorted_cluster.is_empty() {
        let (requests, responses) = tokio::sync::mpsc::unbounded_channel();

        for node in config.cluster.iter() {
            nodes.push(ClusterNode::new(ClusterNodeImpl::new(
                node.as_str(),
                &config.cluster_token,
                requests.clone(),
                config,
            )?));
        }

        Some(RwLock::new(responses))
    } else {
        None
    };

    Ok(Cluster::new(ClusterImpl {
        index,
        nodes,
        raft,
        responses,
        resync,
        snapshot_in_flight,
        poisoned,
        resync_in_progress: Arc::new(Mutex::new(std::collections::HashSet::new())),
        executed,
    }))
}

async fn start_cluster(
    cluster: Cluster,
    shutdown_signal: Arc<AtomicBool>,
    config: Config,
) -> ServerResult<()> {
    if cluster.nodes.is_empty() {
        return Ok(());
    }

    let index = cluster.index;

    for (node_index, node) in cluster.nodes.iter().enumerate() {
        let node = node.clone();
        let shutdown_signal = shutdown_signal.clone();
        tokio::spawn(async move {
            while !shutdown_signal.load(Ordering::Relaxed) {
                if let Some(request) = node.requests_receiver.write().await.recv().await {
                    if let Some(response) = node.send(&request).await {
                        node.record_peer_state(&response);
                        match node.responses.send((request, response)) {
                            Ok(_) => {}
                            Err(e) => crate::warn!(
                                "[{index}] Error sending response to cluster node '{node_index}': {e:?}"
                            ),
                        }
                    } else if request.is_append() || request.resync_dbs().is_some() {
                        let fail_response = raft::Response::new(
                            request.index,
                            raft::ResponseType::CommitError("send failed".into()),
                        );
                        let _ = node.responses.send((request, fail_response));
                    }
                } else {
                    break;
                }
            }

            ServerResult::Ok(())
        });
    }

    let responses_shutdown_signal = shutdown_signal.clone();
    let response_cluster = cluster.clone();
    tokio::spawn(async move {
        while !responses_shutdown_signal.load(Ordering::Relaxed) {
            if let Some((request, response)) = response_cluster
                .responses
                .as_ref()
                .expect("responses is initialized")
                .write()
                .await
                .recv()
                .await
            {
                if let Some(requests) = response_cluster
                    .raft
                    .write()
                    .await
                    .response(&request, &response)
                    .await?
                {
                    for request in requests {
                        let target = request.target;
                        let _ = response_cluster.nodes[request.target as usize]
                            .requests_sender
                            .send(request)
                            .inspect_err(|e| {
                                crate::warn!(
                                    "[{index}] Error sending follow up request to node '{target}': {e:?}"
                                )
                            });
                    }
                }
            } else {
                break;
            }
        }
        ServerResult::Ok(())
    });

    let mut resync_retry_at: Option<Instant> = None;
    let mut panic_resync_done = false;

    while !shutdown_signal.load(Ordering::Relaxed) {
        let is_poisoned = cluster.poisoned.load(Ordering::Relaxed);

        if (cluster.raft.read().await.needs_resync() || is_poisoned)
            && !cluster.resync.load(Ordering::Relaxed)
            && resync_retry_at.map(|t| Instant::now() >= t).unwrap_or(true)
        {
            if is_poisoned {
                if panic_resync_done {
                    crate::error!(
                        "[{index}] Repeated worker panic after resync, shutting down for restart"
                    );
                    shutdown_signal.store(true, Ordering::Relaxed);
                    break;
                }

                let leader = cluster.raft.read().await.leader();
                if leader == Some(index as u64) {
                    crate::error!(
                        "[{index}] Worker panic on leader node, shutting down for restart"
                    );
                    shutdown_signal.store(true, Ordering::Relaxed);
                    break;
                }
            }

            crate::warn!("[{index}] Node needs resync, initiating resync from leader");

            match crate::resync::resync_from_leader(&cluster, &config, is_poisoned).await {
                Ok(_) => {
                    cluster.raft.write().await.clear_needs_resync();
                    if is_poisoned {
                        panic_resync_done = true;
                    }
                }
                Err(e) => {
                    resync_retry_at = Some(Instant::now() + Duration::from_secs(5));
                    crate::error!("[{index}] Resync attempt failed: {e:?}");
                }
            }
        }

        if let Some(requests) = cluster.raft.write().await.process() {
            for request in requests {
                let target = request.target;
                let _ = cluster.nodes[request.target as usize]
                    .requests_sender
                    .send(request)
                    .inspect_err(|e| {
                        crate::warn!(
                            "[{index}] Error sending new request to node '{target}': {e:?}"
                        )
                    });
            }
        } else {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    Ok(())
}

pub(crate) async fn start_with_shutdown(
    cluster: Cluster,
    config: Config,
    mut shutdown_receiver: broadcast::Receiver<()>,
) {
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let cluster_handle = tokio::spawn(start_cluster(
        cluster.clone(),
        shutdown_signal.clone(),
        config,
    ));

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_receiver.recv() => {},
    }

    shutdown_signal.store(true, Ordering::Relaxed);
    let _ = cluster_handle.await;
}

type ExecTask = (Log<ClusterAction>, Option<ResultNotifier>);

pub(crate) struct ClusterStorage {
    result_notifiers: HashMap<DbId, ResultNotifier>,

    exec_sender: tokio::sync::mpsc::UnboundedSender<ExecTask>,
    exec_worker: Option<tokio::task::JoinHandle<()>>,
    node: usize,
    log_body_limit: usize,
    pub(crate) executed: watch::Sender<u64>,
    pub(crate) index: u64,
    pub(crate) term: u64,
    pub(crate) commit: u64,
    pub(crate) prune_index: u64,
    pub(crate) max_log_entries: u64,
    pub(crate) snapshot_in_flight: Arc<AtomicUsize>,
    pub(crate) snapshot_lock: Arc<RwLock<()>>,
    pub(crate) poisoned: Arc<AtomicBool>,
    pub(crate) db: ServerDb,
    pub(crate) cluster_log: ClusterLog,
    pub(crate) db_pool: DbPool,
    pub(crate) failed_indices: Arc<StdRwLock<Vec<u64>>>,
}

impl ClusterStorage {
    #[allow(clippy::too_many_arguments)]
    async fn new(
        node: usize,
        log_body_limit: u64,
        db: ServerDb,
        cluster_log: ClusterLog,
        db_pool: DbPool,
        max_log_entries: u64,
        snapshot_in_flight: Arc<AtomicUsize>,
        poisoned: Arc<AtomicBool>,
    ) -> ServerResult<Self> {
        let (index, term, commit) = cluster_log.cluster_log().await?;
        let logs = cluster_log.logs_unexecuted(commit).await?;
        let (exec_sender, exec_rx) = tokio::sync::mpsc::unbounded_channel();
        let snapshot_lock = Arc::new(RwLock::new(()));
        let executed = watch::Sender::new(executed_before(&logs, commit));
        let failed = cluster_log.failed_indices().await?;

        let mut storage = Self {
            result_notifiers: HashMap::new(),
            executed,
            exec_sender,
            exec_worker: None,
            node,
            log_body_limit: log_body_limit as usize,
            index,
            term,
            commit,
            prune_index: commit.saturating_sub(max_log_entries),
            max_log_entries,
            snapshot_in_flight,
            snapshot_lock,
            poisoned,
            db,
            cluster_log,
            db_pool,
            failed_indices: Arc::new(StdRwLock::new(failed)),
        };

        storage.start_exec_worker(exec_rx);

        for log in logs {
            storage.execute_log(log);
        }

        Ok(storage)
    }

    fn spawn_exec_worker(
        mut exec_rx: UnboundedReceiver<ExecTask>,
        ctx: ExecContext,
        poisoned: Arc<AtomicBool>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some((log, result_notifier)) = exec_rx.recv().await {
                if let Err(e) = std::panic::AssertUnwindSafe(ctx.run(log, result_notifier))
                    .catch_unwind()
                    .await
                {
                    crate::error!("Log execution panicked, stopping worker: {e:?}");
                    poisoned.store(true, Ordering::Relaxed);
                    break;
                }
            }
        })
    }

    fn start_exec_worker(&mut self, exec_rx: UnboundedReceiver<ExecTask>) {
        let ctx = ExecContext {
            node: self.node,
            log_body_limit: self.log_body_limit,
            snapshot_lock: self.snapshot_lock.clone(),
            db: self.db.clone(),
            db_pool: self.db_pool.clone(),
            cluster_log: self.cluster_log.clone(),
            executed: self.executed.clone(),
            failed_indices: self.failed_indices.clone(),
        };
        self.exec_worker = Some(Self::spawn_exec_worker(exec_rx, ctx, self.poisoned.clone()));
    }

    pub(crate) async fn reinit(&mut self, config: &Config) -> ServerResult<()> {
        let (exec_sender, exec_rx) = tokio::sync::mpsc::unbounded_channel();
        self.exec_sender = exec_sender;
        if let Some(handle) = self.exec_worker.take() {
            let _ = handle.await;
        }

        let db_path = format!("{}/{}", config.data_dir, SERVER_DB_FILE);
        let new_db = agdb::Db::new(&db_path)?;
        *self.db.db.write().await = new_db;

        let log_path = format!("{}/{}", config.data_dir, CLUSTER_LOG_FILE);
        let new_log_db = agdb::Db::new(&log_path)?;
        *self.cluster_log.0.write().await = new_log_db;

        let (index, term, commit) = self.cluster_log.cluster_log().await?;
        let logs = self.cluster_log.logs_unexecuted(commit).await?;

        self.index = index;
        self.term = term;
        self.commit = commit;
        self.prune_index = commit.saturating_sub(self.max_log_entries);
        // Indices covered by the snapshot are never executed locally, so waiters must be released here.
        advance_executed(&self.executed, executed_before(&logs, commit));
        self.result_notifiers.clear();
        *self
            .failed_indices
            .write()
            .unwrap_or_else(|e| e.into_inner()) = self.cluster_log.failed_indices().await?;

        self.db_pool.reload(&self.db).await?;
        self.poisoned.store(false, Ordering::Relaxed);
        self.start_exec_worker(exec_rx);

        for log in logs {
            self.execute_log(log);
        }

        Ok(())
    }

    pub(crate) async fn close_handles(&mut self, temp_dir: &Path) -> ServerResult<()> {
        let _snapshot_guard = self.snapshot_lock.write().await;
        self.db_pool.clear().await;
        swap_db(&self.db.db, temp_dir, "server").await?;
        swap_db(&self.cluster_log.0, temp_dir, "log").await?;

        Ok(())
    }

    fn execute_log(&mut self, log: Log<ClusterAction>) {
        let result_notifier = log.db_id.and_then(|id| self.result_notifiers.remove(&id));
        if let Err(e) = self.exec_sender.send((log, result_notifier)) {
            crate::error!(
                "Exec worker is dead, log entry {} skipped: {}",
                e.0.0.index,
                e
            );
        }
    }
}

fn error_chain(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();

    while let Some(e) = source {
        message.push_str(": ");
        message.push_str(&e.to_string());
        source = e.source();
    }

    message
}

fn executed_before(unexecuted: &[Log<ClusterAction>], commit: u64) -> u64 {
    unexecuted
        .first()
        .map_or(commit, |log| log.index.saturating_sub(1))
}

fn advance_executed(executed: &watch::Sender<u64>, index: u64) {
    executed.send_if_modified(|current| {
        if index > *current {
            *current = index;
            true
        } else {
            false
        }
    });
}

fn truncate(s: String, limit: usize) -> String {
    if s.len() <= limit {
        s
    } else {
        let end = (0..=limit)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0);
        let mut t = s;
        t.truncate(end);
        t.push_str("...");
        t
    }
}

struct ExecContext {
    node: usize,
    log_body_limit: usize,
    snapshot_lock: Arc<RwLock<()>>,
    db: ServerDb,
    db_pool: DbPool,
    cluster_log: ClusterLog,
    executed: watch::Sender<u64>,
    failed_indices: Arc<StdRwLock<Vec<u64>>>,
}

impl ExecContext {
    async fn run(&self, log: Log<ClusterAction>, result_notifier: Option<ResultNotifier>) {
        let _snapshot_guard = self.snapshot_lock.read().await;
        let action_name = log.data.name();
        let log_index = log.index;
        let log_id = log.db_id.expect("log should have db_id");
        let now = std::time::Instant::now();
        let result = log.data.exec(self.db.clone(), self.db_pool.clone()).await;
        let duration_us = now.elapsed().as_micros();

        let success = result.is_ok();

        let show_details = self.log_body_limit > 0
            && (crate::logger::debug_enabled() || (!success && crate::logger::warn_enabled()));
        let limit = self.log_body_limit;
        let result_detail = if show_details {
            result
                .as_ref()
                .ok()
                .filter(|r| !matches!(r, ClusterActionResult::None))
                .map(|r| truncate(r.summary(), limit))
        } else {
            None
        };
        let error_detail = if show_details {
            result
                .as_ref()
                .err()
                .map(|e| truncate(e.description.clone(), limit))
        } else {
            None
        };

        advance_executed(&self.executed, log_index);

        let mut notes: Vec<String> = Vec::new();

        if !success {
            match self.cluster_log.set_log_failed(log_id).await {
                Ok(()) => {
                    self.failed_indices
                        .write()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(log_index);
                }
                Err(e) => {
                    notes.push(format!("set_log_failed: {e:?} — flag lost"));
                }
            }
        }

        if let Err(e) = self.cluster_log.log_executed(log_id).await {
            notes.push(format!("log_executed failed: {e:?}"));
        }

        if let Some(rs) = result_notifier
            && rs.send(result.map(|r| (log_index, r))).is_err()
        {
            notes.push("result receiver dropped".to_string());
        }

        crate::logger::log_exec(
            self.node,
            success,
            action_name,
            log_index,
            duration_us,
            result_detail,
            error_detail,
            &notes,
        );
    }
}

async fn swap_db(db: &Arc<RwLock<agdb::Db>>, temp_dir: &Path, name: &str) -> ServerResult<()> {
    let placeholder_path = temp_dir.join(format!("_placeholder_{name}.agdb"));
    let placeholder = agdb::Db::new(&placeholder_path.to_string_lossy())?;
    *db.write().await = placeholder;
    Ok(())
}

impl Storage<ClusterAction, ResultNotifier> for ClusterStorage {
    async fn append(
        &mut self,
        log: Log<ClusterAction>,
        notifier: Option<ResultNotifier>,
    ) -> ServerResult<()> {
        self.cluster_log.remove_uncommitted_logs(log.index).await?;
        let log_id = self.cluster_log.append_log(&log).await?;
        self.index = log.index;
        self.term = log.term;

        if let Some(notifier) = notifier {
            self.result_notifiers.insert(log_id, notifier);
        }

        Ok(())
    }

    async fn commit(&mut self, index: u64) -> ServerResult<()> {
        for log in self.cluster_log.logs_uncommitted(index).await? {
            self.commit = index;
            let log_id = log.db_id.expect("log should have db_id");
            self.cluster_log.log_committed(log_id).await?;
            self.execute_log(log);
        }

        Ok(())
    }

    async fn prune(&mut self, up_to_index: u64) -> ServerResult<()> {
        let up_to_index = std::cmp::min(up_to_index, self.index.saturating_sub(1));
        let ceiling = up_to_index.max(self.prune_index);
        if ceiling > 0 && self.snapshot_in_flight.load(Ordering::Acquire) == 0 {
            self.cluster_log.prune(ceiling).await?;
            self.prune_index = ceiling;
            self.failed_indices
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .retain(|&idx| idx > ceiling);
        }
        Ok(())
    }

    fn log_commit(&self) -> u64 {
        self.commit
    }

    fn log_index(&self) -> u64 {
        self.index
    }

    fn log_term(&self) -> u64 {
        self.term
    }

    fn prune_index(&self) -> u64 {
        self.prune_index
    }

    async fn logs(&self, from_index: u64) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.cluster_log.logs_since(from_index).await
    }

    fn local_failed_indices(&self) -> Vec<u64> {
        self.failed_indices
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    async fn clear_failed_up_to(&mut self, up_to: u64) {
        let to_clear: Vec<u64> = self
            .failed_indices
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter(|&&idx| idx <= up_to)
            .copied()
            .collect();
        if !to_clear.is_empty() {
            if let Err(e) = self.cluster_log.clear_log_failed(&to_clear).await {
                crate::error!("Failed to clear LOG_FAILED up to {}: {:?}", up_to, e);
            }
            self.failed_indices
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .retain(|&idx| idx > up_to);
        }
    }

    async fn resolve_resync_targets(&self, indices: &[u64]) -> Vec<(String, String)> {
        let mut dbs = Vec::new();
        for &idx in indices {
            if let Ok(Some(action)) = self.cluster_log.action_at_index(idx).await {
                for target in action.resync_targets() {
                    let pair = match target {
                        ResyncTarget::UserDb(o, d) => (o.to_string(), d.to_string()),
                        ResyncTarget::ServerDb => (
                            RESYNC_SERVER_DB_OWNER.to_string(),
                            RESYNC_SERVER_DB_NAME.to_string(),
                        ),
                    };
                    if !dbs.contains(&pair) {
                        dbs.push(pair);
                    }
                }
            }
        }
        dbs
    }
}

impl ClusterStorage {
    /// Remove failed indices that map to the given resync target (owner, db).
    /// Called after a successful per-DB resync to prevent re-detection.
    pub(crate) async fn clear_resolved_indices(&self, owner: &str, db: &str) {
        let indices: Vec<u64> = self
            .failed_indices
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let mut to_remove = Vec::new();

        for idx in &indices {
            if let Ok(Some(action)) = self.cluster_log.action_at_index(*idx).await {
                let matches = action.resync_targets().iter().any(|target| match target {
                    ResyncTarget::UserDb(o, d) => *o == owner && *d == db,
                    ResyncTarget::ServerDb => {
                        owner == RESYNC_SERVER_DB_OWNER && db == RESYNC_SERVER_DB_NAME
                    }
                });
                if matches {
                    to_remove.push(*idx);
                }
            }
        }

        if !to_remove.is_empty() {
            // Clear persistent LOG_FAILED markers so a restart does not
            // re-populate these indices into failed_indices.
            if let Err(e) = self.cluster_log.clear_log_failed(&to_remove).await {
                crate::error!(
                    "Failed to clear persistent LOG_FAILED for indices {:?}: {:?}",
                    to_remove,
                    e
                );
            }

            let mut guard = self
                .failed_indices
                .write()
                .unwrap_or_else(|e| e.into_inner());
            guard.retain(|idx| !to_remove.contains(idx));
        }
    }
}

#[cfg(feature = "tls")]
pub(crate) fn root_ca(config: &Config) -> ServerResult<Option<reqwest::Certificate>> {
    static ROOT_CA: std::sync::OnceLock<Option<reqwest::Certificate>> = std::sync::OnceLock::new();

    Ok(ROOT_CA
        .get_or_init(|| {
            if config.tls_root.is_empty() {
                return None;
            }

            let cert_data = std::fs::read(Path::new(&config.tls_root))
                .expect("root certificate could not be read");
            let cert = reqwest::Certificate::from_pem(&cert_data)
                .expect("root certificate data is invalid");
            Some(cert)
        })
        .clone())
}

#[cfg(feature = "tls")]
pub(crate) fn reqwest_client(config: &Config) -> ServerResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(60));

    if let Some(root_ca) = root_ca(config)? {
        builder = builder.add_root_certificate(root_ca).use_rustls_tls();
    }

    Ok(builder.build()?)
}

#[cfg(not(feature = "tls"))]
pub(crate) fn reqwest_client(_config: &Config) -> ServerResult<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    fn node(address: &str) -> ClusterNodeImpl {
        let config = Config::new(crate::config::default_config());
        let (responses, _) = tokio::sync::mpsc::unbounded_channel();
        ClusterNodeImpl::new(address, "token", responses, &config).unwrap()
    }

    fn request() -> AxumRequest {
        AxumRequest::builder()
            .method("POST")
            .uri("/api/v1/db/admin/db/exec_mut")
            .body(Body::from("[]"))
            .unwrap()
    }

    async fn body_text(response: AxumResponse) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).to_string()
    }

    #[tokio::test]
    async fn forward_connect_failure_is_service_unavailable() {
        // GIVEN
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let start = std::time::Instant::now();

        // WHEN
        let response = node(&address)
            .forward(request(), 1, usize::MAX)
            .await
            .unwrap_err();

        // THEN
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(
            start.elapsed() >= FORWARD_CONNECT_BACKOFF * 3,
            "retries with backoff before giving up"
        );
        assert!(body_text(response).await.contains("failed to connect"));
    }

    #[tokio::test]
    async fn forward_failure_after_send_is_bad_gateway() {
        // GIVEN
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        let accepts = tokio::spawn(async move {
            let mut count = 0;
            while let Ok(Ok((mut stream, _))) =
                tokio::time::timeout(Duration::from_millis(500), listener.accept()).await
            {
                count += 1;
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf).await;
            }
            count
        });

        // WHEN
        let response = node(&address)
            .forward(request(), 1, usize::MAX)
            .await
            .unwrap_err();

        // THEN
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert!(
            body_text(response)
                .await
                .contains("forward to leader failed")
        );
        assert_eq!(
            accepts.await.unwrap(),
            1,
            "request that may have been executed is not resent"
        );
    }

    #[tokio::test]
    async fn forward_oversized_body_is_payload_too_large() {
        // GIVEN
        let body_limit = 1;

        // WHEN
        let response = node("http://127.0.0.1:1")
            .forward(request(), 1, body_limit)
            .await
            .unwrap_err();

        // THEN
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn executed_index_only_advances() {
        // GIVEN
        let executed = watch::Sender::new(5);
        let receiver = executed.subscribe();

        // WHEN
        advance_executed(&executed, 3);
        let after_lower = *receiver.borrow();
        advance_executed(&executed, 9);

        // THEN
        assert_eq!(after_lower, 5, "lower index is ignored");
        assert_eq!(*receiver.borrow(), 9, "higher index is applied");
    }

    #[test]
    fn executed_before_first_unexecuted_log() {
        let log = |index| Log {
            db_id: None,
            index,
            term: 1,
            data: ClusterAction::RemoveAllTokens(
                crate::action::remove_all_tokens::RemoveAllTokens {},
            ),
        };

        assert_eq!(executed_before(&[], 10), 10, "all committed logs executed");
        assert_eq!(
            executed_before(&[log(7), log(8)], 10),
            6,
            "first unexecuted is pending"
        );
    }
}
