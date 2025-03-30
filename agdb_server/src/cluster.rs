use crate::action::Action;
use crate::action::ClusterAction;
use crate::action::ClusterActionResult;
use crate::config::Config;
use crate::db_pool::DbPool;
use crate::raft;
use crate::raft::Log;
use crate::raft::Request;
use crate::raft::Response;
use crate::raft::Storage;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use agdb::DbId;
use agdb::StableHash;
use agdb_api::HttpClient;
use agdb_api::ReqwestClient;
use axum::body::Body;
use axum::extract::Request as AxumRequest;
use axum::response::Response as AxumResponse;
use reqwest::StatusCode;
use std::collections::HashMap;
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
        let url = format!("{}{path_query}", self.base_url);

        let mut response = self
            .client
            .client
            .request(
                reqwest::Method::from_str(parts.method.as_str())
                    .map_err(|e| Self::bad_request(&e.to_string()))?,
                url,
            )
            .headers(parts.headers)
            .header("forwarded-by", local_index)
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
            .post(&self.url, &Some(request), &self.token)
            .await
        {
            Ok((_, response)) => Some(response),
            Err(e) => {
                tracing::warn!(
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

pub(crate) async fn new(config: &Config, db: &ServerDb, db_pool: &DbPool) -> ServerResult<Cluster> {
    let index = config
        .cluster
        .iter()
        .position(|url| url == &config.address)
        .unwrap_or_default();
    let mut sorted_cluster: Vec<String> =
        config.cluster.iter().map(|url| url.to_string()).collect();
    sorted_cluster.sort();
    let hash = sorted_cluster.stable_hash();
    let storage = ClusterStorage::new(db.clone(), db_pool.clone()).await?;
    let settings = raft::ClusterSettings {
        index: index as u64,
        hash,
        size: std::cmp::max(config.cluster.len() as u64, 1),
        election_factor: 1,
        heartbeat_timeout: Duration::from_millis(config.cluster_heartbeat_timeout_ms),
        term_timeout: Duration::from_millis(config.cluster_term_timeout_ms),
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
    }))
}

async fn start_cluster(cluster: Cluster, shutdown_signal: Arc<AtomicBool>) -> ServerResult<()> {
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
                            Err(e) => tracing::warn!(
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
                                tracing::warn!(
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

    while !shutdown_signal.load(Ordering::Relaxed) {
        if let Some(requests) = cluster.raft.write().await.process() {
            for request in requests {
                let target = request.target;
                let _ = cluster.nodes[request.target as usize]
                    .requests_sender
                    .send(request)
                    .inspect_err(|e| {
                        tracing::warn!(
                            "[{index}] Error sending new request to node '{target}': {e:?}"
                        )
                    });
            }
        }
    }

    Ok(())
}

pub(crate) async fn start_with_shutdown(
    cluster: Cluster,
    mut shutdown_receiver: broadcast::Receiver<()>,
) {
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let cluster_handle = tokio::spawn(start_cluster(cluster.clone(), shutdown_signal.clone()));

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = shutdown_receiver.recv() => {},
    }

    shutdown_signal.store(true, Ordering::Relaxed);
    let _ = cluster_handle.await;
}

pub(crate) struct ClusterStorage {
    result_notifiers: HashMap<DbId, ResultNotifier>,
    notifier: tokio::sync::broadcast::Sender<u64>,
    index: u64,
    term: u64,
    commit: u64,
    db: ServerDb,
    db_pool: DbPool,
}

impl ClusterStorage {
    async fn new(db: ServerDb, db_pool: DbPool) -> ServerResult<Self> {
        let (index, term, commit) = db.cluster_log().await?;
        let logs = db.logs_unexecuted(commit).await?;

        let mut storage = Self {
            result_notifiers: HashMap::new(),
            notifier: tokio::sync::broadcast::channel(100).0,
            index,
            term,
            commit,
            db,
            db_pool,
        };

        for log in logs {
            storage.execute_log(log).await?;
        }

        Ok(storage)
    }

    async fn execute_log(&mut self, log: Log<ClusterAction>) -> ServerResult<()> {
        let log_id = log.db_id.unwrap_or_default();
        let db = self.db.clone();
        let db_pool = self.db_pool.clone();
        let notifier = self.notifier.clone();
        let result_notifier = self.result_notifiers.remove(&log_id);

        tokio::spawn(async move {
            let result = log.data.exec(db.clone(), db_pool).await;
            let _ = notifier.send(log.index);
            let _ = db.log_executed(log_id).await;

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
        self.db.remove_uncommitted_logs(log.index).await?;
        let log_id = self.db.append_log(&log).await?;
        self.index = log.index;
        self.term = log.term;

        if let Some(notifier) = notifier {
            self.result_notifiers.insert(log_id, notifier);
        }

        Ok(())
    }

    async fn commit(&mut self, index: u64) -> ServerResult<()> {
        for log in self.db.logs_uncommitted(index).await? {
            self.commit = index;
            self.db
                .log_committed(log.db_id.expect("log should have db_id"))
                .await?;
            self.execute_log(log).await?;
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

    async fn logs(&self, from_index: u64) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.db.logs_since(from_index).await
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

            let cert_data = std::fs::read(std::path::Path::new(&config.tls_root))
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
