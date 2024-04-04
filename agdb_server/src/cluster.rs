use crate::config::Config;
use crate::server_error::ServerResult;
use agdb::StableHash;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;

pub(crate) type Cluster = Arc<ClusterImpl>;

type ClusterApi = AgdbApi<ReqwestClient>;

#[allow(dead_code)]
pub(crate) struct ClusterImpl {
    nodes: Vec<ClusterApi>,
    cluster_hash: u64,
}

pub(crate) fn new(config: &Config) -> ServerResult<Cluster> {
    let mut nodes = vec![];

    for node in &config.cluster {
        if node != &format!("{}:{}", config.host, config.port) {
            let (host, port) = node.split_once(':').ok_or(format!(
                "Invalid cluster node address (must be host:port): {}",
                node
            ))?;
            nodes.push(ClusterApi::new(ReqwestClient::new(), host, port.parse()?));
        }
    }

    let mut sorted_cluster = config.cluster.clone();
    sorted_cluster.sort();
    let cluster_hash = sorted_cluster.stable_hash();

    Ok(Cluster::new(ClusterImpl {
        nodes,
        cluster_hash,
    }))
}

async fn start_cluster(cluster: Cluster, shutdown_signal: Arc<AtomicBool>) -> ServerResult<()> {
    if cluster.nodes.is_empty() {
        return Ok(());
    }

    while !shutdown_signal.load(Ordering::Relaxed) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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
