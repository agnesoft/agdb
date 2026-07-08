use crate::action::Action;
use crate::action::ClusterAction;
use crate::action::ClusterActionResult;
use crate::cluster_log::CLUSTER_LOG_FILE;
use crate::cluster_log::ClusterLog;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::raft;
use crate::raft::Log;
use crate::raft::Request;
use crate::raft::Response;
use crate::raft::Storage;
use crate::server_db::SERVER_DB_FILE;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::DbId;
use agdb::StableHash;
use agdb_api::HttpClient;
use agdb_api::ReqwestClient;
use axum::body::Body;
use axum::extract::Request as AxumRequest;
use axum::http::HeaderMap;
use axum::response::Response as AxumResponse;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::signal;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) type Cluster = Arc<ClusterImpl>;

type ClusterNode = Arc<ClusterNodeImpl>;
type ResultNotifier = tokio::sync::oneshot::Sender<ServerResult<(u64, ClusterActionResult)>>;
type ClusterResponseReceiver = UnboundedReceiver<(Request<ClusterAction>, Response)>;

pub(crate) struct ClusterNodeImpl {
    client: ReqwestClient,
    url: String,
    base_url: String,
    base_path: String,
    token: Option<String>,
    requests_sender: UnboundedSender<Request<ClusterAction>>,
    requests_receiver: RwLock<UnboundedReceiver<Request<ClusterAction>>>,
    responses: UnboundedSender<(Request<ClusterAction>, Response)>,
}

pub(crate) struct ClusterImpl {
    pub(crate) index: usize,
    pub(crate) nodes: Vec<ClusterNode>,
    pub(crate) raft: Arc<RwLock<raft::Cluster<ClusterAction, ResultNotifier, ClusterStorage>>>,
    pub(crate) responses: Option<RwLock<ClusterResponseReceiver>>,
    pub(crate) resync: Arc<AtomicBool>,
    pub(crate) snapshot_in_flight: Arc<AtomicBool>,
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
        })
    }

    fn bad_request(message: &str) -> AxumResponse {
        AxumResponse::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(message.to_owned().into())
            .expect("bad request")
    }

    pub(crate) async fn forward(
        &self,
        axum_request: AxumRequest,
        local_index: usize,
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

        let mut response = self
            .client
            .client
            .request(
                reqwest::Method::from_str(parts.method.as_str())
                    .map_err(|e| Self::bad_request(&e.to_string()))?,
                url,
            )
            .headers(headers)
            .body(reqwest::Body::wrap_stream(body.into_data_stream()))
            .send()
            .await
            .map_err(|e| Self::bad_request(&e.to_string()))?;

        let mut axum_response = AxumResponse::builder().status(response.status());

        if let Some(headers) = axum_response.headers_mut() {
            std::mem::swap(headers, response.headers_mut())
        }

        axum_response
            .body(Body::from_stream(response.bytes_stream()))
            .map_err(|e| Self::bad_request(&e.to_string()))
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
    let snapshot_in_flight = Arc::new(AtomicBool::new(false));
    let storage = ClusterStorage::new(
        db.clone(),
        cluster_log.clone(),
        db_pool.clone(),
        config.cluster_max_log_entries,
        snapshot_in_flight.clone(),
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
                        match node.responses.send((request, response)) {
                            Ok(_) => {}
                            Err(e) => crate::warn!(
                                "[{index}] Error sending response to cluster node '{node_index}': {e:?}"
                            ),
                        };
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

    let mut resync_retry_at: Option<tokio::time::Instant> = None;

    while !shutdown_signal.load(Ordering::Relaxed) {
        if cluster.raft.read().await.needs_resync()
            && !cluster.resync.load(Ordering::Relaxed)
            && resync_retry_at
                .map(|t| tokio::time::Instant::now() >= t)
                .unwrap_or(true)
        {
            crate::warn!("[{index}] Node is too far behind, initiating resync from leader");

            match resync_from_leader(&cluster, &config).await {
                Ok(_) => {
                    cluster.raft.write().await.clear_needs_resync();
                }
                Err(e) => {
                    crate::error!("[{index}] Resync attempt failed: {e:?}");
                    resync_retry_at = Some(tokio::time::Instant::now() + Duration::from_secs(5));
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

async fn resync_from_leader(cluster: &Cluster, config: &Config) -> ServerResult<()> {
    let leader = cluster.raft.read().await.leader();
    let leader_index = match leader {
        Some(l) if l as usize != cluster.index => l as usize,
        _ => {
            return Err(ServerError::from(
                "Cannot resync: no leader available or this node is the leader",
            ));
        }
    };

    let mut snapshot_sources: Vec<usize> = (0..cluster.nodes.len())
        .filter(|index| *index != cluster.index && *index != leader_index)
        .collect();
    snapshot_sources.push(leader_index);

    crate::info!(
        "[{}] Starting resync, snapshot candidates: {:?}",
        cluster.index,
        snapshot_sources
    );

    cluster.resync.store(true, Ordering::Relaxed);

    let mut result = Err(ServerError::from("no snapshot source available"));

    for source_index in snapshot_sources {
        crate::info!(
            "[{}] Attempting snapshot download from node {}",
            cluster.index,
            source_index
        );

        match do_resync(cluster, config, source_index).await {
            Ok(()) => {
                result = Ok(());
                break;
            }
            Err(error) => {
                crate::warn!(
                    "[{}] Snapshot download from node {} failed: {:?}",
                    cluster.index,
                    source_index,
                    error
                );
                result = Err(error);
            }
        }
    }

    cluster.resync.store(false, Ordering::Relaxed);

    match &result {
        Ok(()) => crate::info!("[{}] Resync completed successfully", cluster.index),
        Err(e) => crate::error!("[{}] Resync failed: {:?}", cluster.index, e),
    }

    result
}

async fn do_resync(cluster: &Cluster, config: &Config, node_index: usize) -> ServerResult<()> {
    let data = download_snapshot(cluster, config, node_index).await?;
    let snapshot = parse_snapshot(&data)?;

    let data_dir = Path::new(&config.data_dir);
    let backup_dir = data_dir.with_file_name(format!(
        "{}_bak",
        data_dir.file_name().unwrap_or_default().to_string_lossy()
    ));

    replace_data_dir(data_dir, &backup_dir, &snapshot.files)?;
    let mut raft = cluster.raft.write().await;
    raft.storage.reinit(config).await?;
    raft.refresh_local_from_storage();
    drop(raft);

    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }

    Ok(())
}

struct Snapshot {
    files: Vec<(String, Vec<u8>)>,
}

async fn download_snapshot(
    cluster: &Cluster,
    config: &Config,
    node_index: usize,
) -> ServerResult<Vec<u8>> {
    let snapshot_url = format!(
        "{}/api/v1/cluster/snapshot",
        cluster.nodes[node_index].base_url
    );

    let response = cluster.nodes[node_index]
        .client
        .client
        .get(&snapshot_url)
        .bearer_auth(&config.cluster_token)
        .timeout(std::time::Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| ServerError::from(format!("Snapshot download failed: {e:?}")))?;

    if !response.status().is_success() {
        return Err(ServerError::from(format!(
            "Snapshot endpoint returned {}",
            response.status()
        )));
    }

    Ok(response
        .bytes()
        .await
        .map_err(|e| ServerError::from(format!("Failed to read snapshot body: {e:?}")))?
        .to_vec())
}

fn parse_snapshot(data: &[u8]) -> ServerResult<Snapshot> {
    if data.len() < 32 {
        return Err(ServerError::from("Snapshot too small (missing header)"));
    }

    // Header: [log_index(8), log_term(8), log_commit(8), file_count(8)]
    // log_index/term/commit are informational — actual state is read from the files on reinit
    let file_count = u64::from_le_bytes(data[24..32].try_into().unwrap());

    let mut offset = 32;
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for _ in 0..file_count {
        if offset + 4 > data.len() {
            return Err(ServerError::from("Snapshot truncated (path_len)"));
        }
        let path_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + path_len > data.len() {
            return Err(ServerError::from("Snapshot truncated (path)"));
        }
        let path = String::from_utf8_lossy(&data[offset..offset + path_len]).to_string();
        offset += path_len;

        if offset + 8 > data.len() {
            return Err(ServerError::from("Snapshot truncated (file_len)"));
        }
        let file_len = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;

        if offset + file_len > data.len() {
            return Err(ServerError::from("Snapshot truncated (file_data)"));
        }
        let file_data = data[offset..offset + file_len].to_vec();
        offset += file_len;

        files.push((path, file_data));
    }

    Ok(Snapshot { files })
}

fn replace_data_dir(
    data_dir: &Path,
    backup_dir: &Path,
    files: &[(String, Vec<u8>)],
) -> ServerResult<()> {
    if backup_dir.exists() {
        std::fs::remove_dir_all(backup_dir)?;
    }

    if data_dir.exists() {
        std::fs::rename(data_dir, backup_dir)?;
    }

    std::fs::create_dir_all(data_dir)?;

    let canonical_data_dir = data_dir.canonicalize()?;

    for (path, file_data) in files {
        let file_path = canonical_data_dir.join(path);
        let canonical_file_path =
            file_path
                .components()
                .fold(std::path::PathBuf::new(), |mut acc, c| {
                    match c {
                        std::path::Component::ParentDir => {
                            acc.pop();
                        }
                        _ => acc.push(c),
                    }
                    acc
                });

        if !canonical_file_path.starts_with(&canonical_data_dir) {
            return Err(ServerError::from(format!(
                "Invalid path in snapshot: {path}"
            )));
        }

        if let Some(parent) = canonical_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&canonical_file_path, file_data)?;
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

pub(crate) struct ClusterStorage {
    result_notifiers: HashMap<DbId, ResultNotifier>,
    pub(crate) notifier: tokio::sync::broadcast::Sender<u64>,
    pub(crate) index: u64,
    pub(crate) term: u64,
    pub(crate) commit: u64,
    pub(crate) prune_index: u64,
    pub(crate) max_log_entries: u64,
    pub(crate) snapshot_in_flight: Arc<AtomicBool>,
    pub(crate) snapshot_lock: Arc<RwLock<()>>,
    pub(crate) db: ServerDb,
    pub(crate) cluster_log: ClusterLog,
    pub(crate) db_pool: DbPool,
}

impl ClusterStorage {
    async fn new(
        db: ServerDb,
        cluster_log: ClusterLog,
        db_pool: DbPool,
        max_log_entries: u64,
        snapshot_in_flight: Arc<AtomicBool>,
    ) -> ServerResult<Self> {
        let (index, term, commit) = cluster_log.cluster_log().await?;
        let logs = cluster_log.logs_unexecuted(commit).await?;

        let mut storage = Self {
            result_notifiers: HashMap::new(),
            notifier: tokio::sync::broadcast::channel(100).0,
            index,
            term,
            commit,
            prune_index: commit.saturating_sub(max_log_entries),
            max_log_entries,
            snapshot_in_flight,
            snapshot_lock: Arc::new(RwLock::new(())),
            db,
            cluster_log,
            db_pool,
        };

        for log in logs {
            storage.execute_log(log).await?;
        }

        Ok(storage)
    }

    pub(crate) async fn reinit(&mut self, config: &Config) -> ServerResult<()> {
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
        self.result_notifiers.clear();

        self.db_pool.reload(&self.db).await?;

        for log in logs {
            self.execute_log(log).await?;
        }

        Ok(())
    }

    async fn execute_log(&mut self, log: Log<ClusterAction>) -> ServerResult<()> {
        let log_id = log.db_id.unwrap_or_default();
        let db = self.db.clone();
        let db_pool = self.db_pool.clone();
        let cluster_log = self.cluster_log.clone();
        let notifier = self.notifier.clone();
        let result_notifier = self.result_notifiers.remove(&log_id);
        let snapshot_lock = self.snapshot_lock.clone();

        tokio::spawn(async move {
            let _snapshot_guard = snapshot_lock.read().await;
            let result = log.data.exec(db.clone(), db_pool).await;
            let _ = notifier.send(log.index);
            let _ = cluster_log.log_executed(log_id).await;

            if let Some(rs) = result_notifier {
                let _ = rs.send(result.map(|r| (log.index, r)));
            }
        });

        Ok(())
    }

    pub(crate) async fn subscribe(&self) -> tokio::sync::broadcast::Receiver<u64> {
        self.notifier.subscribe()
    }
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
            self.cluster_log
                .log_committed(log.db_id.expect("log should have db_id"))
                .await?;
            self.execute_log(log).await?;
        }

        Ok(())
    }

    async fn prune(&mut self, up_to_index: u64) -> ServerResult<()> {
        let up_to_index = std::cmp::min(up_to_index, self.index.saturating_sub(1));
        if up_to_index > self.prune_index && !self.snapshot_in_flight.load(Ordering::Acquire) {
            self.cluster_log.prune(up_to_index).await?;
            self.prune_index = up_to_index;
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
