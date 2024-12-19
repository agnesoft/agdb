use crate::action::Action;
use crate::action::ClusterAction;
use crate::action::ClusterResponse;
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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

pub(crate) type Cluster = Arc<ClusterImpl>;

type ClusterNode = Arc<ClusterNodeImpl>;
type ResultNotifier = tokio::sync::oneshot::Sender<ServerResult<ClusterResponse>>;
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
    pub(crate) async fn append<T: Action + Into<ClusterAction>>(
        &self,
        action: T,
    ) -> ServerResult<ClusterResponse> {
        let (sender, receiver) = tokio::sync::oneshot::channel::<ServerResult<ClusterResponse>>();
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
    ) -> Self {
        let base = if address.starts_with("http") || address.starts_with("https") {
            address.to_string()
        } else {
            format!("http://{address}")
        };

        let (requests_sender, requests_receiver) = tokio::sync::mpsc::unbounded_channel();

        Self {
            client: ReqwestClient::new(),
            url: format!("{base}api/v1/cluster"),
            base_url: base.trim_end_matches("/").to_string(),
            token: Some(token.to_string()),
            requests_sender,
            requests_receiver: RwLock::new(requests_receiver),
            responses,
        }
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
                    "[{}] Error sending request to cluster node {}: {:?}",
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
    let storage = ClusterStorage::new(db.clone(), db_pool.clone(), config.clone()).await?;
    let settings = raft::ClusterSettings {
        index: index as u64,
        hash,
        size: std::cmp::max(config.cluster.len() as u64, 1),
        election_factor: 1,
        heartbeat_timeout: Duration::from_secs(1),
        term_timeout: Duration::from_secs(3),
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
            )));
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

    for node in &cluster.nodes {
        let node = node.clone();
        let shutdown_signal = shutdown_signal.clone();
        tokio::spawn(async move {
            while !shutdown_signal.load(Ordering::Relaxed) {
                if let Some(request) = node.requests_receiver.write().await.recv().await {
                    if let Some(response) = node.send(&request).await {
                        node.responses.send((request, response))?;
                    }
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
                        response_cluster.nodes[request.target as usize]
                            .requests_sender
                            .send(request)?;
                    }
                };
            }
        }
        ServerResult::Ok(())
    });

    while !shutdown_signal.load(Ordering::Relaxed) {
        if let Some(requests) = cluster.raft.write().await.process() {
            for request in requests {
                cluster.nodes[request.target as usize]
                    .requests_sender
                    .send(request)?;
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
    notifiers: HashMap<DbId, ResultNotifier>,
    index: u64,
    term: u64,
    commit: u64,
    executed: Arc<AtomicU64>,
    db: ServerDb,
    db_pool: DbPool,
    config: Config,
}

impl ClusterStorage {
    async fn new(db: ServerDb, db_pool: DbPool, config: Config) -> ServerResult<Self> {
        let (index, term, commit) = db.cluster_log().await?;
        let logs = db.logs_unexecuted_until(commit).await?;

        let mut storage = Self {
            notifiers: HashMap::new(),
            index,
            term,
            commit,
            executed: Arc::new(AtomicU64::new(index)),
            db,
            db_pool,
            config,
        };

        for log in logs {
            storage.execute_log(log).await?;
        }

        Ok(storage)
    }

    async fn execute_log(&mut self, log: Log<ClusterAction>) -> ServerResult<()> {
        let log_id = log.db_id.unwrap_or_default();
        let executed = self.executed.clone();
        let mut db = self.db.clone();
        let mut db_pool = self.db_pool.clone();
        let config = self.config.clone();
        let notifier = self.notifiers.remove(&log_id);

        tokio::spawn(async move {
            let result = log.data.exec(&mut db, &mut db_pool, &config).await;
            executed.fetch_max(log.index, Ordering::Relaxed);
            let _ = db.log_executed(log_id).await;

            if let Some(notifier) = notifier {
                let _ = notifier.send(result);
            }
        });

        Ok(())
    }
}

impl Storage<ClusterAction, ResultNotifier> for ClusterStorage {
    async fn append(
        &mut self,
        log: Log<ClusterAction>,
        notifier: Option<ResultNotifier>,
    ) -> ServerResult<()> {
        self.db.remove_unexecuted_logs_since(log.index).await?;
        let log_id = self.db.append_log(&log).await?;
        self.index = log.index;
        self.term = log.term;

        if let Some(notifier) = notifier {
            self.notifiers.insert(log_id, notifier);
        }

        Ok(())
    }

    async fn commit(&mut self, index: u64) -> ServerResult<()> {
        for log in self.db.logs_unexecuted_until(index).await? {
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

    fn log_executed(&self) -> u64 {
        self.executed.load(Ordering::Relaxed)
    }

    fn log_index(&self) -> u64 {
        self.index
    }

    fn log_term(&self) -> u64 {
        self.term
    }

    async fn logs(&self, since_index: u64) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.db.logs_since(since_index).await
    }
}
