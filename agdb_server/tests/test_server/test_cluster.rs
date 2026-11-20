use super::TestServerImpl;
use super::api_for_test;
use super::cluster::start_cluster;
use super::shared;
use super::test_error::TestError;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_LOG_ENTRIES;

pub use super::cluster::wait_for_leader;

/// The leader is the first node.
pub struct TestCluster {
    pub addresses: Vec<String>,
    pub data_dirs: Vec<String>,
    _nodes: Vec<TestServerImpl>,
}

impl TestCluster {
    /// The shared cluster of the test run.
    pub async fn new() -> Result<Self, TestError> {
        Ok(Self {
            addresses: shared().cluster_addresses.clone(),
            data_dirs: shared().cluster_data_dirs.clone(),
            _nodes: Vec::new(),
        })
    }

    pub async fn new_private() -> Result<Self, TestError> {
        let nodes = create_cluster(3, false).await?;

        Ok(Self {
            addresses: nodes.iter().map(|node| node.address.clone()).collect(),
            data_dirs: nodes.iter().map(|node| node.data_dir.clone()).collect(),
            _nodes: nodes,
        })
    }

    pub fn leader(&self) -> AgdbApi<ReqwestClient> {
        api_for_test(&self.addresses[0])
    }

    pub fn follower(&self) -> AgdbApi<ReqwestClient> {
        api_for_test(&self.addresses[1])
    }

    pub fn data_dir(&self, address: &str) -> String {
        let node = self
            .addresses
            .iter()
            .position(|node| node == address)
            .unwrap_or_else(|| panic!("{address} is not a node of the cluster"));
        self.data_dirs[node].clone()
    }
}

pub async fn create_cluster(nodes: usize, tls: bool) -> Result<Vec<TestServerImpl>, TestError> {
    start_cluster(nodes, tls, DEFAULT_CLUSTER_MAX_LOG_ENTRIES).await
}

pub async fn create_cluster_with_max_log_entries(
    nodes: usize,
    max_log_entries: u64,
) -> Result<Vec<TestServerImpl>, TestError> {
    start_cluster(nodes, false, max_log_entries).await
}
