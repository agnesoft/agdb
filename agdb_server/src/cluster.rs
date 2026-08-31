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
use futures::StreamExt;
use reqwest::StatusCode;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
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
    pub(crate) snapshot_in_flight: Arc<AtomicUsize>,
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

    #[allow(clippy::result_large_err)]
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
    let snapshot_in_flight = Arc::new(AtomicUsize::new(0));
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
                    resync_retry_at = Some(tokio::time::Instant::now() + Duration::from_secs(5));
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

const SNAPSHOT_PARTIAL_TTL_SECS: u64 = 3600;
const SNAPSHOT_EXTRACT_CHUNK: usize = 65536;

async fn validate_snapshot_header(partial_file: &Path, current_commit: u64) -> ServerResult<()> {
    let mut file = tokio::fs::File::open(partial_file).await?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header)
        .await
        .map_err(|_| ServerError::from("snapshot header missing or truncated"))?;
    let header_commit = u64::from_le_bytes(header[16..24].try_into().unwrap());
    if header_commit < current_commit {
        return Err(ServerError::from(format!(
            "snapshot commit {header_commit} is behind current commit {current_commit}"
        )));
    }
    Ok(())
}

async fn do_resync(cluster: &Cluster, config: &Config, node_index: usize) -> ServerResult<()> {
    let data_dir = Path::new(&config.data_dir);
    let dir_name = data_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let partial_dir = data_dir.with_file_name(format!("{dir_name}.snapshot_partial"));
    let install_dir = data_dir.with_file_name(format!("{dir_name}.snapshot_install"));
    let backup_dir = data_dir.with_file_name(format!("{dir_name}.snapshot_bak"));

    if install_dir.exists() {
        let _ = std::fs::remove_dir_all(&install_dir);
    }

    cleanup_stale_partial(&partial_dir);
    download_snapshot_to_partial(cluster, config, node_index, &partial_dir).await?;

    let partial_file = partial_dir.join("data.bin");
    let current_commit = cluster.raft.read().await.storage.commit;
    validate_snapshot_header(&partial_file, current_commit).await?;

    if let Err(e) = extract_snapshot_binary(&partial_file, &install_dir).await {
        let _ = std::fs::remove_dir_all(&install_dir);
        return Err(e);
    }

    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }

    let mut raft = cluster.raft.write().await;
    raft.storage.close_handles(&partial_dir).await?;

    if data_dir.exists() {
        std::fs::rename(data_dir, &backup_dir)?;
    }

    if let Err(e) = std::fs::rename(&install_dir, data_dir) {
        let _ = std::fs::rename(&backup_dir, data_dir);
        return Err(ServerError::from(format!(
            "failed to install snapshot: {e}"
        )));
    }

    if let Err(e) = raft.storage.reinit(config).await {
        let _ = std::fs::remove_dir_all(data_dir);
        let _ = std::fs::rename(&backup_dir, data_dir);
        let _ = std::fs::remove_dir_all(&partial_dir);
        drop(raft);
        return Err(e);
    }

    raft.refresh_local_from_storage();
    drop(raft);

    let _ = std::fs::remove_dir_all(&backup_dir);
    let _ = std::fs::remove_dir_all(&partial_dir);

    Ok(())
}

async fn download_snapshot_to_partial(
    cluster: &Cluster,
    config: &Config,
    node_index: usize,
    partial_dir: &Path,
) -> ServerResult<()> {
    std::fs::create_dir_all(partial_dir)?;
    let partial_file = partial_dir.join("data.bin");
    let id_file = partial_dir.join(".id");

    let (resume_offset, resume_id) = if partial_file.exists() && id_file.exists() {
        let offset = partial_file.metadata()?.len();
        let id = std::fs::read_to_string(&id_file).unwrap_or_default();
        (offset, id.trim().to_string())
    } else {
        (0u64, String::new())
    };

    let snapshot_url = format!(
        "{}/api/v1/cluster/snapshot",
        cluster.nodes[node_index].base_url
    );

    let mut request = cluster.nodes[node_index]
        .client
        .client
        .get(&snapshot_url)
        .bearer_auth(&config.cluster_token)
        .timeout(Duration::from_secs(600));

    if resume_offset > 0 && !resume_id.is_empty() {
        request = request
            .header("range", format!("bytes={resume_offset}-"))
            .header("x-snapshot-resume-id", &resume_id);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ServerError::from(format!("snapshot request failed: {e:?}")))?;

    let http_status = response.status().as_u16();
    if http_status != 200 && http_status != 206 {
        return Err(ServerError::from(format!(
            "snapshot endpoint returned {http_status}"
        )));
    }

    let server_id = response
        .headers()
        .get("x-snapshot-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let is_resume = http_status == 206 && !server_id.is_empty() && server_id == resume_id;

    if !is_resume {
        let _ = std::fs::remove_file(&partial_file);
    }

    std::fs::write(&id_file, &server_id)?;

    let append = is_resume && partial_file.exists();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(&partial_file)
        .await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ServerError::from(format!("stream read error: {e:?}")))?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(())
}

async fn extract_snapshot_binary(partial_file: &Path, install_dir: &Path) -> ServerResult<()> {
    std::fs::create_dir_all(install_dir)?;

    let mut file = tokio::fs::File::open(partial_file).await?;

    let mut header = [0u8; 32];
    file.read_exact(&mut header)
        .await
        .map_err(|_| ServerError::from("snapshot too small (missing header)"))?;

    let file_count = u64::from_le_bytes(header[24..32].try_into().unwrap());

    const MAX_SNAPSHOT_FILES: u64 = 100_000;

    if file_count > MAX_SNAPSHOT_FILES {
        return Err(ServerError::from(format!(
            "snapshot file_count {file_count} exceeds sanity limit {MAX_SNAPSHOT_FILES}"
        )));
    }

    let canonical_install = install_dir.canonicalize()?;
    let mut buf = vec![0u8; SNAPSHOT_EXTRACT_CHUNK];

    for _ in 0..file_count {
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)
            .await
            .map_err(|_| ServerError::from("snapshot truncated (path_len)"))?;
        let path_len = u32::from_le_bytes(len_buf) as usize;

        if path_len == 0 || path_len > 4096 {
            return Err(ServerError::from(format!(
                "invalid path length in snapshot: {path_len}"
            )));
        }

        let mut path_buf = vec![0u8; path_len];
        file.read_exact(&mut path_buf)
            .await
            .map_err(|_| ServerError::from("snapshot truncated (path)"))?;
        let rel_path = String::from_utf8(path_buf)
            .map_err(|e| ServerError::from(format!("invalid path encoding: {e}")))?;

        let mut file_len_buf = [0u8; 8];
        file.read_exact(&mut file_len_buf)
            .await
            .map_err(|_| ServerError::from("snapshot truncated (file_len)"))?;
        let file_len = u64::from_le_bytes(file_len_buf);

        let abs = canonical_install.join(&rel_path);
        if abs
            .components()
            .any(|c| c == std::path::Component::ParentDir)
            || !abs.starts_with(&canonical_install)
        {
            return Err(ServerError::from(format!(
                "invalid path in snapshot: {rel_path}"
            )));
        }

        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out = tokio::fs::File::create(&abs).await?;
        let mut remaining = file_len;

        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
            file.read_exact(&mut buf[..to_read])
                .await
                .map_err(|_| ServerError::from("snapshot truncated (file_data)"))?;
            out.write_all(&buf[..to_read]).await?;
            remaining -= to_read as u64;
        }
        out.sync_data().await?;
    }

    Ok(())
}

fn cleanup_stale_partial(partial_dir: &Path) {
    if !partial_dir.exists() {
        return;
    }
    let Ok(metadata) = partial_dir.metadata() else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(age) = std::time::SystemTime::now().duration_since(modified) else {
        return;
    };
    if age.as_secs() >= SNAPSHOT_PARTIAL_TTL_SECS {
        let _ = std::fs::remove_dir_all(partial_dir);
    }
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
    pub(crate) snapshot_in_flight: Arc<AtomicUsize>,
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
        snapshot_in_flight: Arc<AtomicUsize>,
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

    pub(crate) async fn close_handles(&mut self, temp_dir: &Path) -> ServerResult<()> {
        let _snapshot_guard = self.snapshot_lock.write().await;
        self.db_pool.clear().await;
        swap_db(&self.db.db, temp_dir, "server").await?;
        swap_db(&self.cluster_log.0, temp_dir, "log").await?;

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
            self.cluster_log
                .log_committed(log.db_id.expect("log should have db_id"))
                .await?;
            self.execute_log(log).await?;
        }

        Ok(())
    }

    async fn prune(&mut self, up_to_index: u64) -> ServerResult<()> {
        let up_to_index = std::cmp::min(up_to_index, self.index.saturating_sub(1));
        // Use the higher of the incoming index and the existing ceiling so that entries
        // skipped on a prior call (because their async execute_log task had not yet removed
        // the EXECUTED key) are retried on every subsequent heartbeat
        let ceiling = up_to_index.max(self.prune_index);
        if ceiling > 0 && self.snapshot_in_flight.load(Ordering::Acquire) == 0 {
            self.cluster_log.prune(ceiling).await?;
            self.prune_index = ceiling;
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
