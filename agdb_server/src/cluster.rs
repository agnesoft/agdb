use crate::config::Config;
use crate::raft;
use crate::raft::Log;
use crate::raft::Request;
use crate::raft::Response;
use crate::raft::Storage;
use crate::server_db::ServerDb;
use crate::server_error::ServerResult;
use agdb::StableHash;
use agdb_api::HttpClient;
use agdb_api::ReqwestClient;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
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

pub(crate) struct ClusterNodeImpl {
    client: ReqwestClient,
    url: String,
    token: Option<String>,
    requests_sender: UnboundedSender<Request>,
    requests_receiver: RwLock<UnboundedReceiver<Request>>,
    responses: UnboundedSender<(Request, Response)>,
}

pub(crate) struct ClusterImpl {
    pub(crate) index: usize,
    pub(crate) nodes: Vec<ClusterNode>,
    pub(crate) raft: Arc<RwLock<raft::Cluster<ClusterStorage>>>,
    pub(crate) responses: RwLock<UnboundedReceiver<(Request, Response)>>,
}

impl ClusterNodeImpl {
    fn new(address: &str, token: &str, responses: UnboundedSender<(Request, Response)>) -> Self {
        let base = if address.starts_with("http") || address.starts_with("https") {
            address.to_string()
        } else {
            format!("http://{address}")
        };

        let (requests_sender, requests_receiver) = tokio::sync::mpsc::unbounded_channel();

        Self {
            client: ReqwestClient::new(),
            url: format!("{base}api/v1/cluster"),
            token: Some(token.to_string()),
            requests_sender,
            requests_receiver: RwLock::new(requests_receiver),
            responses,
        }
    }

    async fn send(&self, request: &raft::Request) -> ServerResult<raft::Response> {
        Ok(self
            .client
            .post(&self.url, &Some(request), &self.token)
            .await?
            .1)
    }
}

pub(crate) async fn new(config: &Config, db: &ServerDb) -> ServerResult<Cluster> {
    let mut nodes = vec![];
    let mut sorted_cluster: Vec<String> =
        config.cluster.iter().map(|url| url.to_string()).collect();
    sorted_cluster.sort();
    let index = config
        .cluster
        .iter()
        .position(|url| url == &config.address)
        .unwrap();
    let hash = sorted_cluster.stable_hash();
    let storage = ClusterStorage::new(db.clone()).await?;
    let settings = raft::ClusterSettings {
        index: index as u64,
        hash,
        size: config.cluster.len() as u64,
        election_factor: 1,
        heartbeat_timeout: Duration::from_secs(1),
        term_timeout: Duration::from_secs(3),
    };
    let raft = Arc::new(RwLock::new(raft::Cluster::new(storage, settings)));
    let (requests, responses) = tokio::sync::mpsc::unbounded_channel();

    for node in config.cluster.iter() {
        nodes.push(ClusterNode::new(ClusterNodeImpl::new(
            node.as_str(),
            &config.cluster_token,
            requests.clone(),
        )));
    }

    Ok(Cluster::new(ClusterImpl {
        index,
        nodes,
        raft,
        responses: RwLock::new(responses),
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
                    let response = node.send(&request).await?;
                    node.responses.send((request, response))?;
                }
            }

            ServerResult::Ok(())
        });
    }

    let responses_shutdown_signal = shutdown_signal.clone();
    let response_cluster = cluster.clone();
    tokio::spawn(async move {
        while !responses_shutdown_signal.load(Ordering::Relaxed) {
            if let Some((request, response)) = response_cluster.responses.write().await.recv().await
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

pub(crate) async fn append(cluster: Cluster, data: Vec<u8>) -> ServerResult<()> {
    for request in cluster.raft.write().await.append(data).await {
        cluster.nodes[request.target as usize]
            .requests_sender
            .send(request)?;
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
    logs: VecDeque<Log>,
    index: u64,
    term: u64,
    commit: u64,
    db: ServerDb,
}

impl ClusterStorage {
    async fn new(db: ServerDb) -> ServerResult<Self> {
        let (index, term, commit) = db.cluster_log().await?;
        Ok(Self {
            logs: VecDeque::new(),
            index,
            term,
            commit,
            db,
        })
    }
}

impl Storage for ClusterStorage {
    async fn append(&mut self, log: Log) {
        if let Some(index) = self
            .logs
            .iter()
            .rev()
            .position(|l| l.index == log.index && l.term == log.term)
        {
            self.logs.truncate(index);
        }

        self.index = log.index;
        self.term = log.term;
        self.logs.push_back(log);
    }

    async fn commit(&mut self, index: u64) -> ServerResult<()> {
        while let Some(log) = self.logs.pop_front() {
            if log.index > index {
                self.logs.push_front(log);
                break;
            } else {
                self.commit = log.index;
                self.db.commit_log(&log).await?;
                //TODO: Execute action
            }
        }
        Ok(())
    }

    fn current_index(&self) -> u64 {
        self.index
    }

    fn current_term(&self) -> u64 {
        self.term
    }

    fn current_commit(&self) -> u64 {
        self.commit
    }

    async fn logs(&self, since_index: u64) -> ServerResult<Vec<Log>> {
        let mut logs = self.db.logs(since_index).await?;
        logs.extend_from_slice(&self.logs.iter().cloned().collect::<Vec<Log>>());
        Ok(logs)
    }
}
