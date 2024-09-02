use crate::config::Config;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb::StableHash;
use agdb_api::HttpClient;
use agdb_api::ReqwestClient;
use axum::http::StatusCode;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

const TERM_TIMEOUT: Duration = Duration::from_secs(3);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(1);
const LOOP_TIMEOUT: Duration = Duration::from_millis(10);

pub(crate) type Cluster = Arc<ClusterImpl>;

type ClusterNode = Arc<ClusterNodeImpl>;

pub(crate) struct ClusterNodeImpl {
    client: ReqwestClient,
    base_url: String,
    token: Option<String>,
    index: usize,
}

#[derive(Copy, Clone)]
pub(crate) enum ClusterState {
    Leader,
    LeaderElect,
    Follower(usize),
    Candidate,
    Election,
}

pub(crate) struct ClusterData {
    pub(crate) state: ClusterState,
    pub(crate) timer: std::time::Instant,
    pub(crate) term: u64,
    pub(crate) voted: u64,
    pub(crate) leader: Arc<AtomicBool>,
}

pub(crate) struct ClusterImpl {
    pub(crate) nodes: Vec<ClusterNode>,
    pub(crate) cluster_hash: u64,
    pub(crate) index: usize,
    pub(crate) data: RwLock<ClusterData>,
}

impl ClusterImpl {
    pub(crate) async fn leader(&self) -> Option<usize> {
        match self.data.read().await.state {
            ClusterState::Leader | ClusterState::LeaderElect => Some(self.index),
            ClusterState::Follower(leader) => Some(leader),
            ClusterState::Candidate | ClusterState::Election => None,
        }
    }
}

impl ClusterNodeImpl {
    fn new(address: &str, token: &str, index: usize) -> Self {
        let base = if address.starts_with("http") || address.starts_with("https") {
            address.to_string()
        } else {
            format!("http://{address}")
        };

        Self {
            client: ReqwestClient::new(),
            base_url: format!("{base}api/v1"),
            token: Some(token.to_string()),
            index,
        }
    }

    async fn heartbeat(
        &self,
        cluster_hash: u64,
        term: u64,
        leader: usize,
    ) -> ServerResult<(u16, String)> {
        self.client
            .post::<(), String>(
                &self.url(&format!(
                    "/cluster/heartbeat?cluster_hash={cluster_hash}&term={term}&leader={leader}"
                )),
                &None,
                &self.token,
            )
            .await
            .map_err(|e| {
                ServerError::new(
                    StatusCode::from_u16(e.status).unwrap_or(StatusCode::NOT_IMPLEMENTED),
                    &e.description,
                )
            })
    }

    async fn vote(
        &self,
        cluster_hash: u64,
        term: u64,
        leader: usize,
    ) -> ServerResult<(u16, String)> {
        self.client
            .get::<String>(
                &self.url(&format!(
                    "/cluster/vote?cluster_hash={cluster_hash}&term={term}&leader={leader}"
                )),
                &self.token,
            )
            .await
            .map_err(|e| {
                ServerError::new(
                    StatusCode::from_u16(e.status).unwrap_or(StatusCode::NOT_IMPLEMENTED),
                    &e.description,
                )
            })
    }

    fn url(&self, uri: &str) -> String {
        format!("{}{uri}", self.base_url)
    }
}

pub(crate) fn new(config: &Config) -> ServerResult<Cluster> {
    let mut nodes = vec![];
    let mut index = 0;

    for (i, node) in config.cluster.iter().enumerate() {
        if node == &config.address {
            index = i;
        } else {
            nodes.push(ClusterNode::new(ClusterNodeImpl::new(
                node.as_str(),
                &config.cluster_token,
                i,
            )));
        }
    }

    let mut sorted_cluster: Vec<String> =
        config.cluster.iter().map(|url| url.to_string()).collect();
    sorted_cluster.sort();
    let cluster_hash = sorted_cluster.stable_hash();

    Ok(Cluster::new(ClusterImpl {
        nodes,
        cluster_hash,
        index,
        data: RwLock::new(ClusterData {
            timer: Instant::now(),
            state: ClusterState::Election,
            term: 0,
            voted: 0,
            leader: Arc::new(AtomicBool::new(false)),
        }),
    }))
}

async fn start_cluster(cluster: Cluster, shutdown_signal: Arc<AtomicBool>) -> ServerResult<()> {
    if cluster.nodes.len() < 2 {
        return Ok(());
    }

    while !shutdown_signal.load(Ordering::Relaxed) {
        let timer;
        let state;

        {
            let data = cluster.data.read().await;
            timer = data.timer;
            state = data.state;
        }

        match state {
            ClusterState::LeaderElect => heartbeat(&cluster, shutdown_signal.clone()).await?,
            ClusterState::Follower(_) => {
                if timer.elapsed() > TERM_TIMEOUT {
                    let mut data = cluster.data.write().await;
                    data.state = ClusterState::Election;
                    data.timer = Instant::now();
                }
            }
            ClusterState::Election => {
                if timer.elapsed() >= Duration::from_secs(cluster.index as u64) {
                    election(&cluster).await?;
                }
            }
            ClusterState::Candidate | ClusterState::Leader => {}
        }

        std::thread::sleep(LOOP_TIMEOUT);
    }

    Ok(())
}

async fn heartbeat(cluster: &Cluster, shutdown_signal: Arc<AtomicBool>) -> ServerResult<()> {
    cluster.data.write().await.state = ClusterState::Leader;

    let term = cluster.data.read().await.term;
    let cluster_hash = cluster.cluster_hash;
    let leader = cluster.index;
    let is_leader = cluster.data.read().await.leader.clone();
    let cluster_index = cluster.index;

    for node in &cluster.nodes {
        let node = node.clone();
        let is_leader = is_leader.clone();
        let shutdown_signal = shutdown_signal.clone();

        tokio::spawn(async move {
            while is_leader.load(Ordering::Relaxed) && !shutdown_signal.load(Ordering::Relaxed) {
                match node.heartbeat(cluster_hash, term, leader).await {
                    Ok((status, message)) => {
                        if status != 200 {
                            tracing::warn!(
                                "[{cluster_index}] Heartbeat rejected by {}: ({}) {}",
                                node.index,
                                status,
                                message
                            );
                        }
                    }
                    Err(e) => {
                        let message = format!(
                            "[{cluster_index}] Heartbeat error on node {}: ({}) {}",
                            node.index, e.status, e.description
                        );

                        if e.status.is_client_error() {
                            tracing::warn!(message);
                        } else {
                            tracing::error!(message);
                        }
                    }
                }

                std::thread::sleep(HEARTBEAT_TIMEOUT);
            }
        });
    }

    Ok(())
}

async fn election(cluster: &Cluster) -> ServerResult<()> {
    let timer = Instant::now();
    let election_term;

    {
        let mut data = cluster.data.write().await;
        data.state = ClusterState::Candidate;
        election_term = data.term + 1;
        data.term = election_term;
        data.voted = election_term;
    }

    let cluster_hash = cluster.cluster_hash;
    let index = cluster.index;
    let quorum = (cluster.nodes.len() + 1) / 2 + 1;

    tracing::info!("[{index}] Starting election (cluster: {cluster_hash}, term: {election_term}, quorum: {quorum}/{})", cluster.nodes.len() + 1);

    let votes = Arc::new(AtomicUsize::new(1));
    let voted = Arc::new(AtomicUsize::new(1));

    for node in &cluster.nodes {
        let node = node.clone();
        let votes = votes.clone();
        let voted = voted.clone();
        let cluster = cluster.clone();

        tokio::spawn(async move {
            match node.vote(cluster_hash, election_term, index).await {
                Ok(_) => {
                    tracing::info!(
                        "[{}] Vote for term {election_term} ACCEPTED by {}",
                        cluster.index,
                        node.index
                    );

                    if (votes.fetch_add(1, Ordering::Relaxed) + 1) == quorum {
                        tracing::info!(
                            "[{index}] Elected as leader for term {election_term} ({}ms)",
                            timer.elapsed().as_millis()
                        );

                        let mut data = cluster.data.write().await;
                        data.state = ClusterState::LeaderElect;
                        data.leader.store(true, Ordering::Relaxed);
                        data.term = election_term;
                        data.timer = Instant::now();
                    }
                }
                Err(e) => {
                    if e.status.is_client_error() {
                        tracing::warn!(
                            "[{}] Vote for term {election_term} REJECTED by {}: ({}) {}",
                            cluster.index,
                            node.index,
                            e.status,
                            e.description
                        );
                    } else {
                        tracing::error!(
                            "[{}] Vote for term {election_term} FAILED on {}: ({}) {}",
                            cluster.index,
                            node.index,
                            e.status,
                            e.description
                        );
                    }
                }
            }

            if voted.fetch_add(1, Ordering::Relaxed) == cluster.nodes.len() {
                let is_leader = cluster.data.read().await.leader.load(Ordering::Relaxed);

                if !is_leader {
                    tracing::warn!(
                        "[{index}] Election for term {election_term} failed - {}/{} (quorum: {quorum}/{}) ({}ms)",
                        votes.load(Ordering::Relaxed),
                        cluster.nodes.len() + 1,
                        cluster.nodes.len() + 1,
                        timer.elapsed().as_millis(),
                    );

                    let mut data = cluster.data.write().await;
                    data.state = ClusterState::Election;
                    data.timer = Instant::now();
                }
            };
        });
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
