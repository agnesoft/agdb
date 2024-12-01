use std::time::Duration;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub struct Log {
    index: u64,
    term: u64,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct MismatchedValues {
    local: Option<u64>,
    requested: Option<u64>,
}

#[derive(Debug)]
pub struct LogMismatch {
    index: MismatchedValues,
    term: MismatchedValues,
    commit: MismatchedValues,
}

#[derive(Debug)]
pub enum RequestType {
    Append(Vec<Log>),
    Heartbeat,
    Vote,
}

#[derive(Debug)]
pub enum ResponseType {
    Ok,
    ClusterMismatch(MismatchedValues),
    LeaderMismatch(MismatchedValues),
    TermMismatch(MismatchedValues),
    LogMismatch(LogMismatch),
    AlreadyVoted(MismatchedValues),
}

#[derive(Debug)]
enum ClusterState {
    Candidate,
    Election,
    Follower(u64),
    Leader,
    Voted(u64),
}

#[derive(Debug)]
pub struct Request {
    hash: u64,
    index: u64,
    target: u64,
    term: u64,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
    data: RequestType,
}

#[derive(Debug)]
pub struct Response {
    target: u64,
    result: ResponseType,
}

struct Node {
    index: u64,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
    timer: Instant,
    voted: bool,
}

pub trait Storage {
    fn append(&mut self, log: Log);
    fn commit(&mut self, index: u64);
    fn current_index(&self) -> u64;
    fn current_term(&self) -> u64;
    fn current_commit(&self) -> u64;
    fn logs(&self, since_index: u64, since_term: u64) -> Vec<Log>;
}

pub struct Cluster<S: Storage> {
    storage: S,
    nodes: Vec<Node>,
    state: ClusterState,
    hash: u64,
    size: u64,
    index: u64,
    term: u64,
    election_timeout: Duration,
    heartbeat_timeout: Duration,
    term_timeout: Duration,
}

pub struct ClusterSettings {
    pub index: u64,
    pub size: u64,
    pub hash: u64,
    pub election_factor: u64,
    pub heartbeat_timeout: Duration,
    pub term_timeout: Duration,
}

impl<S: Storage> Cluster<S> {
    pub fn new(storage: S, settings: ClusterSettings) -> Self {
        Self {
            state: ClusterState::Election,
            nodes: (0..settings.size)
                .map(|i| Node {
                    index: i,
                    log_index: if i == settings.index {
                        storage.current_index()
                    } else {
                        0
                    },
                    log_term: if i == settings.index {
                        storage.current_term()
                    } else {
                        0
                    },
                    log_commit: if i == settings.index {
                        storage.current_commit()
                    } else {
                        0
                    },
                    timer: Instant::now(),
                    voted: i == settings.index,
                })
                .collect(),
            hash: settings.hash,
            size: settings.size,
            index: settings.index,
            term: 0,
            election_timeout: Duration::from_secs(settings.election_factor * settings.index),
            heartbeat_timeout: settings.heartbeat_timeout,
            term_timeout: settings.term_timeout,
            storage,
        }
    }

    pub fn append(&mut self, log: Vec<u8>) -> Vec<Request> {
        let log = Log {
            index: self.local().log_index,
            term: self.term,
            data: log,
        };
        self.local_mut().log_index += 1;
        self.local_mut().log_term = self.term;
        let requests = self
            .nodes
            .iter()
            .filter(|node| self.index != node.index)
            .map(|node| Request {
                hash: self.hash,
                index: self.index,
                target: node.index,
                term: self.term,
                log_index: self.local().log_index,
                log_term: self.local().log_term,
                log_commit: self.local().log_commit,
                data: RequestType::Append(vec![log.clone()]),
            })
            .collect();
        self.storage.append(log);
        requests
    }

    pub fn process(&mut self) -> Option<Vec<Request>> {
        if let ClusterState::Leader = self.state {
            let requests = self.heartbeat();

            if requests.is_empty() {
                return None;
            }

            requests.iter().for_each(|request| {
                self.node_mut(request.target).timer = Instant::now();
            });

            return Some(requests);
        } else {
            if let ClusterState::Election = self.state {
                if self.local().timer.elapsed() >= self.election_timeout {
                    let requests = self.election();
                    self.local_mut().timer = Instant::now();
                    return Some(requests);
                }
            }

            if self.local().timer.elapsed() > self.term_timeout {
                self.state = ClusterState::Election;
                self.local_mut().timer = Instant::now();
            }
        }

        None
    }

    pub fn request(&mut self, request: &Request) -> Response {
        let response = match request.data {
            RequestType::Append(ref logs) => self.append_request(request, logs),
            RequestType::Heartbeat => self.heartbeat_request(request),
            RequestType::Vote => self.vote_request(request),
        };

        self.node_mut(request.target).timer = Instant::now();

        match response {
            Ok(response) => response,
            Err(response) => response,
        }
    }

    pub fn response(&mut self, request: &Request, response: &Response) -> Option<Vec<Request>> {
        use ClusterState::*;
        use RequestType::*;
        use ResponseType::*;

        match (&self.state, &request.data, &response.result) {
            (Candidate, Vote, Ok) => self.vote_received(request),
            (Leader, Heartbeat | Append(_), Ok) => self.commit(request),
            (Leader, Heartbeat | Append(_), LogMismatch(_)) => self.reconcile(request),
            _ => None,
        }
    }

    fn reconcile(&mut self, request: &Request) -> Option<Vec<Request>> {
        let logs = self.storage.logs(
            self.node(request.target).log_index,
            self.node(request.target).log_term,
        );
        self.node_mut(request.target).timer = Instant::now();
        Some(vec![Request {
            hash: self.hash,
            index: self.index,
            target: request.target,
            term: self.term,
            log_index: self.local().log_index,
            log_term: self.local().log_term,
            log_commit: self.local().log_commit,
            data: RequestType::Append(logs),
        }])
    }

    fn election(&mut self) -> Vec<Request> {
        self.state = ClusterState::Candidate;
        self.nodes
            .iter_mut()
            .filter(|node| self.index != node.index)
            .for_each(|node| {
                node.voted = false;
            });
        self.nodes
            .iter()
            .filter(|node| self.index != node.index)
            .map(|node| Request {
                hash: self.hash,
                index: self.index,
                target: node.index,
                term: self.term + 1,
                log_index: self.local().log_index,
                log_term: self.local().log_term,
                log_commit: self.local().log_commit,
                data: RequestType::Vote,
            })
            .collect()
    }

    fn append_request(&mut self, request: &Request, logs: &[Log]) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_term(request)?;
        self.become_follower(request);

        for log in logs {
            self.validate_log_commit(request, log)?;
            self.append_storage(log);
        }

        if self.local().log_commit < request.log_commit {
            let available_commit = std::cmp::min(self.local().log_index, request.log_commit);
            self.commit_storage(available_commit);
        }

        Self::ok(request)
    }
    fn append_storage(&mut self, log: &Log) {
        self.storage.append(log.clone());
        self.local_mut().log_index = log.index + 1;
        self.local_mut().log_term = log.term;
    }

    fn commit_storage(&mut self, index: u64) {
        self.storage.commit(index);
        self.local_mut().log_commit = index;
    }

    fn commit(&mut self, request: &Request) -> Option<Vec<Request>> {
        self.node_mut(request.target).log_index = request.log_index;
        self.node_mut(request.target).log_term = request.log_term;
        self.node_mut(request.target).log_commit = request.log_commit;

        let quorum = self.size / 2 + 1;

        if self.local().log_commit < request.log_index
            && self
                .nodes
                .iter()
                .filter(|node| node.log_index >= request.log_index)
                .count() as u64
                >= quorum
        {
            self.commit_storage(request.log_index);
            return Some(self.heartbeat_no_timer());
        }

        None
    }

    fn heartbeat(&mut self) -> Vec<Request> {
        self.nodes
            .iter()
            .filter(|node| {
                self.index != node.index
                    && self.node(node.index).timer.elapsed() > self.heartbeat_timeout
            })
            .map(|node| Request {
                hash: self.hash,
                index: self.index,
                target: node.index,
                term: self.term,
                log_index: self.local().log_index,
                log_term: self.local().log_term,
                log_commit: self.local().log_commit,
                data: RequestType::Heartbeat,
            })
            .collect()
    }

    fn heartbeat_no_timer(&mut self) -> Vec<Request> {
        let requests: Vec<Request> = self
            .nodes
            .iter()
            .filter(|node| self.index != node.index)
            .map(|node| Request {
                hash: self.hash,
                index: self.index,
                target: node.index,
                term: self.term,
                log_index: self.local().log_index,
                log_term: self.local().log_term,
                log_commit: self.local().log_commit,
                data: RequestType::Heartbeat,
            })
            .collect();

        requests.iter().for_each(|request| {
            self.node_mut(request.target).timer = Instant::now();
        });

        requests
    }

    fn heartbeat_request(&mut self, request: &Request) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_term(request)?;
        self.become_follower(request);
        self.validate_log(request)?;

        if self.local().log_commit < request.log_commit {
            let available_commit = std::cmp::min(self.local().log_index, request.log_commit);
            self.commit_storage(available_commit);
        }

        Self::ok(request)
    }

    fn become_follower(&mut self, request: &Request) {
        if self.term < request.term {
            self.term = request.term;
            self.state = ClusterState::Follower(request.index);
        }
    }

    fn vote_received(&mut self, request: &Request) -> Option<Vec<Request>> {
        self.node_mut(request.target).voted = true;

        let votes = self.nodes.iter().filter(|node| node.voted).count() as u64;
        let quorum = self.size / 2;

        if votes > quorum {
            self.state = ClusterState::Leader;
            self.term = request.term;
            return Some(self.heartbeat_no_timer());
        }

        None
    }

    fn vote_request(&mut self, request: &Request) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_vote_state(request)?;
        self.validate_term_for_vote(request)?;
        self.validate_log_for_vote(request)?;
        self.state = ClusterState::Voted(request.term);
        Self::ok(request)
    }

    fn validate_hash(&self, request: &Request) -> Result<(), Response> {
        if self.hash != request.hash {
            return Err(Response {
                target: request.index,
                result: ResponseType::ClusterMismatch(MismatchedValues {
                    local: Some(self.hash),
                    requested: Some(request.hash),
                }),
            });
        }

        Ok(())
    }

    fn validate_vote_state(&self, request: &Request) -> Result<(), Response> {
        match self.state {
            ClusterState::Leader | ClusterState::Candidate => Err(Response {
                target: request.index,
                result: ResponseType::LeaderMismatch(MismatchedValues {
                    local: Some(self.index),
                    requested: None,
                }),
            }),
            ClusterState::Follower(leader) => Err(Response {
                target: request.index,
                result: ResponseType::LeaderMismatch(MismatchedValues {
                    local: Some(leader),
                    requested: None,
                }),
            }),
            ClusterState::Voted(term) if request.term <= term => Err(Response {
                target: request.index,
                result: ResponseType::AlreadyVoted(MismatchedValues {
                    local: Some(term),
                    requested: Some(request.term),
                }),
            }),
            _ => Ok(()),
        }
    }

    fn validate_log(&self, request: &Request) -> Result<(), Response> {
        if self.local().log_index != request.log_index || self.local().log_term != request.log_term
        {
            return Err(Response {
                target: request.index,
                result: ResponseType::LogMismatch(LogMismatch {
                    index: MismatchedValues {
                        local: Some(self.local().log_index),
                        requested: Some(request.log_index),
                    },
                    term: MismatchedValues {
                        local: Some(self.local().log_term),
                        requested: Some(request.log_term),
                    },
                    commit: MismatchedValues {
                        local: Some(self.local().log_commit),
                        requested: Some(request.log_commit),
                    },
                }),
            });
        }

        Ok(())
    }

    fn validate_log_commit(&self, request: &Request, log: &Log) -> Result<(), Response> {
        if self.local().log_commit > log.index {
            return Err(Response {
                target: request.index,
                result: ResponseType::LogMismatch(LogMismatch {
                    index: MismatchedValues {
                        local: Some(self.local().log_index),
                        requested: Some(request.log_index),
                    },
                    term: MismatchedValues {
                        local: Some(self.local().log_term),
                        requested: Some(request.log_term),
                    },
                    commit: MismatchedValues {
                        local: Some(self.local().log_commit),
                        requested: Some(request.log_commit),
                    },
                }),
            });
        }

        Ok(())
    }

    fn validate_log_for_vote(&self, request: &Request) -> Result<(), Response> {
        if self.local().log_index > request.log_index
            || self.local().log_term > request.log_term
            || self.local().log_commit > request.log_commit
        {
            return Err(Response {
                target: request.index,
                result: ResponseType::LogMismatch(LogMismatch {
                    index: MismatchedValues {
                        local: Some(self.local().log_index),
                        requested: Some(request.log_index),
                    },
                    term: MismatchedValues {
                        local: Some(self.local().log_term),
                        requested: Some(request.log_term),
                    },
                    commit: MismatchedValues {
                        local: Some(self.local().log_commit),
                        requested: Some(request.log_commit),
                    },
                }),
            });
        }

        Ok(())
    }

    fn validate_current_term(&self, request: &Request) -> Result<(), Response> {
        if self.term != request.term {
            return Err(Response {
                target: request.index,
                result: ResponseType::TermMismatch(MismatchedValues {
                    local: Some(self.term),
                    requested: Some(request.term),
                }),
            });
        }

        Ok(())
    }

    fn validate_term_for_vote(&self, request: &Request) -> Result<(), Response> {
        if self.term >= request.term {
            return Err(Response {
                target: request.index,
                result: ResponseType::TermMismatch(MismatchedValues {
                    local: Some(self.term),
                    requested: Some(request.term),
                }),
            });
        }

        Ok(())
    }

    fn validate_term(&self, request: &Request) -> Result<(), Response> {
        if self.term > request.term {
            return Err(Response {
                target: request.index,
                result: ResponseType::TermMismatch(MismatchedValues {
                    local: Some(self.term),
                    requested: Some(request.term),
                }),
            });
        }

        Ok(())
    }

    fn ok(request: &Request) -> Result<Response, Response> {
        Ok(Response {
            target: request.index,
            result: ResponseType::Ok,
        })
    }

    fn node(&self, index: u64) -> &Node {
        &self.nodes[index as usize]
    }

    fn node_mut(&mut self, index: u64) -> &mut Node {
        &mut self.nodes[index as usize]
    }

    fn local(&self) -> &Node {
        self.node(self.index)
    }

    fn local_mut(&mut self) -> &mut Node {
        self.node_mut(self.index)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::OnceLock;
    use tokio::sync::mpsc::Sender;
    use tokio::sync::RwLock;

    #[derive(Debug, Default, Clone, PartialEq)]
    struct TestStorage {
        logs: Vec<Log>,
        commit: u64,
    }

    struct TestNodeImpl {
        cluster: Cluster<TestStorage>,
    }

    type TestNode = Arc<RwLock<TestNodeImpl>>;

    struct TestCluster {
        nodes: Arc<RwLock<Vec<TestNode>>>,
        shutdown: Arc<AtomicBool>,
        blocked: Arc<AtomicU64>,
        requests_channel: Option<Sender<Request>>,
    }

    impl Storage for TestStorage {
        fn append(&mut self, log: Log) {
            self.logs.truncate(log.index as usize);
            self.logs.push(log);
        }

        fn commit(&mut self, index: u64) {
            self.commit = index;
        }

        fn current_index(&self) -> u64 {
            self.logs.len() as u64
        }

        fn current_term(&self) -> u64 {
            self.logs.last().map(|log| log.term).unwrap_or(0)
        }

        fn current_commit(&self) -> u64 {
            self.commit
        }

        fn logs(&self, index: u64, _term: u64) -> Vec<Log> {
            self.logs[index as usize..].to_vec()
        }
    }

    impl TestCluster {
        fn new(size: u64) -> Self {
            static LOGGER_INIT: OnceLock<()> = OnceLock::new();
            LOGGER_INIT.get_or_init(|| tracing_subscriber::fmt().init());

            let nodes = (0..size)
                .map(|index| {
                    let storage = TestStorage {
                        logs: Vec::new(),
                        commit: 0,
                    };
                    let settings = ClusterSettings {
                        index,
                        size,
                        hash: 123,
                        election_factor: 1,
                        heartbeat_timeout: Duration::from_secs(1),
                        term_timeout: Duration::from_secs(1),
                    };
                    Arc::new(RwLock::new(TestNodeImpl {
                        cluster: Cluster::new(storage, settings),
                    }))
                })
                .collect();

            Self {
                nodes: Arc::new(RwLock::new(nodes)),
                shutdown: Arc::new(AtomicBool::new(false)),
                blocked: Arc::new(AtomicU64::new(99)),
                requests_channel: None,
            }
        }

        async fn start(&mut self) {
            let (requests_channel, mut requests_receiver) =
                tokio::sync::mpsc::channel::<Request>(100);
            self.requests_channel = Some(requests_channel.clone());
            let (responses_channel, mut responses_receiver) = tokio::sync::mpsc::channel(100);
            let shutdown = self.shutdown.clone();
            let nodes = self.nodes.clone();
            let blocked = self.blocked.clone();
            tokio::spawn(async move {
                while !shutdown.load(Ordering::Relaxed) {
                    if let Some(request) = requests_receiver.recv().await {
                        let blocked_node = blocked.load(Ordering::Relaxed);
                        if request.target != blocked_node && request.index != blocked_node {
                            let target = nodes.read().await[request.target as usize].clone();
                            let response = target.write().await.cluster.request(&request);
                            responses_channel.send((request, response)).await?;
                        } else {
                            tracing::info!("Blocked: {:?}", request);
                        }
                    }
                }

                anyhow::Ok(())
            });

            let shutdown = self.shutdown.clone();
            let nodes = self.nodes.clone();
            let req_channel = requests_channel.clone();
            tokio::spawn(async move {
                while !shutdown.load(Ordering::Relaxed) {
                    if let Some((request, response)) = responses_receiver.recv().await {
                        tracing::info!("{:?} -> {:?}", request, response);
                        let origin = nodes.read().await[response.target as usize].clone();
                        let new_requests =
                            origin.write().await.cluster.response(&request, &response);
                        if let Some(new_requests) = new_requests {
                            for req in new_requests {
                                req_channel.send(req).await?;
                            }
                        }
                    }
                }

                anyhow::Ok(())
            });

            for node in self.nodes.read().await.iter() {
                let node = node.clone();
                let shutdown = self.shutdown.clone();
                let req_channel = requests_channel.clone();
                tokio::spawn(async move {
                    while !shutdown.load(Ordering::Relaxed) {
                        if let Some(requests) = node.write().await.cluster.process() {
                            for request in requests {
                                req_channel.send(request).await?;
                            }
                        }

                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }

                    anyhow::Ok(())
                });
            }
        }

        async fn block(&self, index: u64) {
            self.blocked.store(index, Ordering::Relaxed);
        }

        async fn unblock(&self) {
            self.blocked.store(u64::MAX, Ordering::Relaxed);
        }

        async fn expect_leader(&self, index: u64) {
            let timer = Instant::now();
            while timer.elapsed() < TIMEOUT {
                tokio::time::sleep(Duration::from_millis(10)).await;

                if let ClusterState::Leader = self.nodes.read().await[index as usize]
                    .read()
                    .await
                    .cluster
                    .state
                {
                    return;
                }
            }

            panic!("Leader not found within {:?}", TIMEOUT);
        }

        async fn expect_follower(&self, index: u64) {
            let timer = Instant::now();
            while timer.elapsed() < TIMEOUT {
                tokio::time::sleep(Duration::from_millis(10)).await;

                if let ClusterState::Follower(_) = self.nodes.read().await[index as usize]
                    .read()
                    .await
                    .cluster
                    .state
                {
                    return;
                }
            }

            panic!("{index} has not become a followerwithin {:?}", TIMEOUT);
        }

        async fn expect_storage_synced(&self, left: u64, right: u64) {
            let timer = Instant::now();
            let mut left_storage = TestStorage::default();
            let mut right_storage = TestStorage::default();

            while timer.elapsed() < TIMEOUT {
                tokio::time::sleep(Duration::from_millis(10)).await;
                left_storage = self.nodes.read().await[left as usize]
                    .read()
                    .await
                    .cluster
                    .storage
                    .clone();

                right_storage = self.nodes.read().await[right as usize]
                    .read()
                    .await
                    .cluster
                    .storage
                    .clone();

                if left_storage == right_storage {
                    return;
                }
            }

            panic!(
                "{left} is not in sync with {right} in {:?}:\nLEFT\n{:?}\nRIGHT:\n{:?}",
                TIMEOUT, left_storage, right_storage
            );
        }

        async fn append(&self, index: u64, log: Vec<u8>) -> anyhow::Result<()> {
            let requests = self.nodes.read().await[index as usize]
                .write()
                .await
                .cluster
                .append(log);

            for request in requests {
                if let Some(channel) = &self.requests_channel {
                    channel.send(request).await?;
                }
            }

            Ok(())
        }
    }

    impl Drop for TestCluster {
        fn drop(&mut self) {
            self.shutdown.store(true, Ordering::Relaxed);
        }
    }

    const TIMEOUT: Duration = Duration::from_secs(5);

    #[tokio::test]
    async fn rebalance() -> anyhow::Result<()> {
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0).await;
        cluster.block(0).await;
        cluster.expect_leader(1).await;
        cluster.unblock().await;
        cluster.expect_follower(0).await;
        Ok(())
    }

    #[tokio::test]
    async fn replication() -> anyhow::Result<()> {
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0).await;
        cluster.append(0, b"0".to_vec()).await?;
        cluster.expect_storage_synced(0, 1).await;
        cluster.expect_storage_synced(0, 2).await;
        Ok(())
    }

    #[tokio::test]
    async fn reconciliation() -> anyhow::Result<()> {
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0).await;
        cluster.block(1).await;
        cluster.append(0, b"0".to_vec()).await?;
        cluster.expect_storage_synced(0, 2).await;
        cluster.unblock().await;
        cluster.expect_storage_synced(0, 1).await;
        Ok(())
    }

    #[tokio::test]
    async fn reconciliation_multiple_values() -> anyhow::Result<()> {
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0).await;
        cluster.append(0, b"0".to_vec().clone()).await?;
        cluster.expect_storage_synced(0, 1).await;
        cluster.expect_storage_synced(0, 2).await;
        cluster.block(2).await;
        cluster.append(0, b"1".to_vec()).await?;
        cluster.append(0, b"2".to_vec()).await?;
        cluster.expect_storage_synced(0, 1).await;
        cluster.unblock().await;
        cluster.expect_storage_synced(0, 2).await;
        Ok(())
    }

    #[tokio::test]
    async fn drop_uncommited_value() -> anyhow::Result<()> {
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0).await;
        cluster.block(0).await;
        cluster.append(0, b"0".to_vec()).await?;
        cluster.expect_leader(1).await;
        cluster.append(1, b"1".to_vec()).await?;
        cluster.expect_storage_synced(1, 2).await;
        cluster.unblock().await;
        cluster.expect_storage_synced(0, 1).await;
        Ok(())
    }
}
