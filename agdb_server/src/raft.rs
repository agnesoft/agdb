use std::time::Duration;
use std::time::Instant;

const VERSION: u64 = 1;

#[derive(Debug, Clone)]
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
    Commit(u64),
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
    version: u64,
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
    version: u64,
    target: u64,
    result: ResponseType,
}

pub trait Storage {
    fn append(&mut self, log: Log);
    fn commit(&mut self, index: u64);
    fn current_index(&self) -> u64;
    fn current_term(&self) -> u64;
    fn current_commit(&self) -> u64;
    fn logs(&self, index: u64, term: u64) -> Vec<Log>;
}

pub struct Cluster<S: Storage> {
    storage: S,
    nodes: Vec<u64>,
    timers: Vec<Instant>,
    state: ClusterState,
    hash: u64,
    size: u64,
    index: u64,
    term: u64,
    votes: u64,
    log_index: u64,
    log_term: u64,
    log_commit: u64,
    timer: Instant,
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
                .filter(|i| *i != settings.index)
                .collect(),
            timers: vec![Instant::now(); settings.size as usize],
            hash: settings.hash,
            size: settings.size,
            index: settings.index,
            term: 0,
            votes: 0,
            log_index: storage.current_index(),
            log_term: storage.current_term(),
            log_commit: storage.current_commit(),
            timer: Instant::now(),
            election_timeout: Duration::from_secs(settings.election_factor * settings.index),
            heartbeat_timeout: settings.heartbeat_timeout,
            term_timeout: settings.term_timeout,
            storage,
        }
    }

    pub fn append(&mut self, log: Vec<u8>) -> Vec<Request> {
        self.log_index += 1;
        self.log_term = self.term;
        let log = Log {
            index: self.log_index,
            term: self.log_term,
            data: log,
        };
        let requests = self
            .nodes
            .iter()
            .map(|node| Request {
                version: VERSION,
                hash: self.hash,
                index: self.index,
                target: *node,
                term: self.term,
                log_index: self.log_index,
                log_term: self.log_term,
                log_commit: self.log_commit,
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
                self.timers[request.target as usize] = Instant::now();
            });

            return Some(requests);
        } else {
            if let ClusterState::Election = self.state {
                if self.timer.elapsed() >= self.election_timeout {
                    let requests = self.election();
                    self.timer = Instant::now();
                    return Some(requests);
                }
            }

            if self.timer.elapsed() > self.term_timeout {
                self.state = ClusterState::Election;
                self.timer = Instant::now();
            }
        }

        None
    }

    pub fn request(&mut self, request: &Request) -> Response {
        let response = match request.data {
            RequestType::Append(ref logs) => self.append_request(request, logs),
            RequestType::Commit(index) => self.commit_request(request, index),
            RequestType::Heartbeat => self.heartbeat_request(request),
            RequestType::Vote => self.vote_request(request),
        };

        self.timers[request.index as usize] = Instant::now();
        self.timer = Instant::now();

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
            (Candidate, Vote, Ok) => self.vote_ok(request.term),
            (Leader, Heartbeat | Append(_) | Commit(_), LogMismatch(values)) => {
                let logs = self.storage.logs(
                    values.index.local.unwrap_or_default(),
                    values.term.local.unwrap_or_default(),
                );
                self.timers[request.target as usize] = Instant::now();
                Some(vec![Request {
                    version: VERSION,
                    hash: self.hash,
                    index: self.index,
                    target: request.target,
                    term: self.term,
                    log_index: self.index,
                    log_term: self.term,
                    log_commit: self.log_commit,
                    data: Append(logs),
                }])
            }
            _ => None,
        }
    }

    fn election(&mut self) -> Vec<Request> {
        self.state = ClusterState::Candidate;
        self.votes = 1;
        self.nodes
            .iter()
            .map(|node| Request {
                version: VERSION,
                hash: self.hash,
                index: self.index,
                target: *node,
                term: self.term + 1,
                log_index: self.log_index,
                log_term: self.log_term,
                log_commit: self.log_commit,
                data: RequestType::Vote,
            })
            .collect()
    }

    fn append_request(&mut self, request: &Request, logs: &[Log]) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_term(request)?;

        if request.term > self.term {
            self.term = request.term;
            self.state = ClusterState::Follower(request.index);
        }

        for log in logs {
            self.validate_log_commit(request, log)?;
            self.append_storage(log);
        }

        Self::ok(request)
    }
    fn append_storage(&mut self, log: &Log) {
        self.log_index = log.index;
        self.log_term = log.term;
        self.storage.append(log.clone());
    }

    fn commit(&mut self, index: u64) {
        self.log_commit = index;
        self.storage.commit(index);
    }

    fn commit_request(&mut self, request: &Request, index: u64) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_term(request)?;

        if request.term > self.term {
            self.term = request.term;
            self.state = ClusterState::Follower(request.index);
        }

        if self.log_commit < index {
            self.commit(index);
        }

        Self::ok(request)
    }

    fn heartbeat(&mut self) -> Vec<Request> {
        self.nodes
            .iter()
            .filter_map(|node| {
                if self.timers[*node as usize].elapsed() > self.heartbeat_timeout {
                    Some(Request {
                        version: VERSION,
                        hash: self.hash,
                        index: self.index,
                        target: *node,
                        term: self.term,
                        log_index: self.log_index,
                        log_term: self.log_term,
                        log_commit: self.log_commit,
                        data: RequestType::Heartbeat,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn heartbeat_no_timer(&mut self) -> Vec<Request> {
        self.nodes
            .iter()
            .map(|node| Request {
                version: VERSION,
                hash: self.hash,
                index: self.index,
                target: *node,
                term: self.term,
                log_index: self.log_index,
                log_term: self.log_term,
                log_commit: self.log_commit,
                data: RequestType::Heartbeat,
            })
            .collect()
    }

    fn heartbeat_request(&mut self, request: &Request) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_term(request)?;

        if request.term > self.term {
            self.term = request.term;
            self.state = ClusterState::Follower(request.index);
        }

        self.validate_log(request)?;

        if request.log_commit > self.log_commit {
            self.commit(request.log_commit);
        }

        Self::ok(request)
    }

    fn vote_ok(&mut self, term: u64) -> Option<Vec<Request>> {
        self.votes += 1;

        if self.votes > self.size / 2 {
            self.state = ClusterState::Leader;
            self.term = term;
            return Some(self.heartbeat_no_timer());
        }

        None
    }

    fn vote_request(&mut self, request: &Request) -> Result<Response, Response> {
        self.validate_hash(request)?;
        self.validate_no_leader(request)?;
        self.validate_term_for_vote(request)?;
        self.validate_log_for_vote(request)?;
        self.state = ClusterState::Voted(request.index);
        Self::ok(request)
    }

    fn validate_hash(&self, request: &Request) -> Result<(), Response> {
        if request.hash != self.hash {
            return Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::ClusterMismatch(MismatchedValues {
                    local: Some(self.hash),
                    requested: Some(request.hash),
                }),
            });
        }

        Ok(())
    }

    fn validate_no_leader(&self, request: &Request) -> Result<(), Response> {
        match self.state {
            ClusterState::Leader | ClusterState::Candidate => Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LeaderMismatch(MismatchedValues {
                    local: Some(self.index),
                    requested: None,
                }),
            }),
            ClusterState::Follower(leader) => Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LeaderMismatch(MismatchedValues {
                    local: Some(leader),
                    requested: None,
                }),
            }),
            _ => Ok(()),
        }
    }

    fn validate_leader(&self, request: &Request) -> Result<(), Response> {
        match self.state {
            ClusterState::Follower(leader) if request.index == leader => Ok(()),
            ClusterState::Follower(leader) => Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LeaderMismatch(MismatchedValues {
                    local: Some(leader),
                    requested: Some(request.index),
                }),
            }),
            _ => Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LeaderMismatch(MismatchedValues {
                    local: None,
                    requested: Some(request.index),
                }),
            }),
        }
    }

    fn validate_log(&self, request: &Request) -> Result<(), Response> {
        if request.log_index != self.log_index || request.log_term != self.log_term {
            return Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LogMismatch(LogMismatch {
                    index: MismatchedValues {
                        local: Some(self.log_index),
                        requested: Some(request.log_index),
                    },
                    term: MismatchedValues {
                        local: Some(self.log_term),
                        requested: Some(request.log_term),
                    },
                    commit: MismatchedValues {
                        local: Some(self.log_commit),
                        requested: Some(request.log_commit),
                    },
                }),
            });
        }

        Ok(())
    }

    fn validate_log_commit(&self, request: &Request, log: &Log) -> Result<(), Response> {
        if log.index < self.log_commit {
            return Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LogMismatch(LogMismatch {
                    index: MismatchedValues {
                        local: Some(self.log_index),
                        requested: Some(request.log_index),
                    },
                    term: MismatchedValues {
                        local: Some(self.log_term),
                        requested: Some(request.log_term),
                    },
                    commit: MismatchedValues {
                        local: Some(self.log_commit),
                        requested: Some(request.log_commit),
                    },
                }),
            });
        }

        Ok(())
    }

    fn validate_log_for_vote(&self, request: &Request) -> Result<(), Response> {
        if request.log_index < self.log_index
            || request.log_term < self.log_term
            || request.log_commit < self.log_commit
        {
            return Err(Response {
                version: VERSION,
                target: request.index,
                result: ResponseType::LogMismatch(LogMismatch {
                    index: MismatchedValues {
                        local: Some(self.log_index),
                        requested: Some(request.log_index),
                    },
                    term: MismatchedValues {
                        local: Some(self.log_term),
                        requested: Some(request.log_term),
                    },
                    commit: MismatchedValues {
                        local: Some(self.log_commit),
                        requested: Some(request.log_commit),
                    },
                }),
            });
        }

        Ok(())
    }

    fn validate_current_term(&self, request: &Request) -> Result<(), Response> {
        if request.term != self.term {
            return Err(Response {
                version: VERSION,
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
        if request.term <= self.term {
            return Err(Response {
                version: VERSION,
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
        if request.term < self.term {
            return Err(Response {
                version: VERSION,
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
            version: VERSION,
            target: request.index,
            result: ResponseType::Ok,
        })
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

        async fn expect_leader(&self, index: u64, timeout: Duration) {
            let timer = Instant::now();
            while timer.elapsed() < timeout {
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

            panic!("Leader not found within {:?}", timeout);
        }

        async fn expect_follower(&self, index: u64, timeout: Duration) {
            let timer = Instant::now();
            while timer.elapsed() < timeout {
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

            panic!("{index} has not become a followerwithin {:?}", timeout);
        }

        async fn expect_value(&self, index: u64, value: Vec<u8>, timeout: Duration) {
            let timer = Instant::now();
            while timer.elapsed() < timeout {
                tokio::time::sleep(Duration::from_millis(10)).await;

                if self.nodes.read().await[index as usize]
                    .read()
                    .await
                    .cluster
                    .storage
                    .logs
                    .iter()
                    .any(|log| log.data == value)
                {
                    return;
                }
            }

            panic!("{index} has not received the value within {:?}", timeout);
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
        cluster.expect_leader(0, TIMEOUT).await;
        cluster.block(0).await;
        cluster.expect_leader(1, TIMEOUT).await;
        cluster.unblock().await;
        cluster.expect_follower(0, TIMEOUT).await;
        Ok(())
    }

    #[tokio::test]
    async fn replication() -> anyhow::Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0, TIMEOUT).await;
        let value = b"0".to_vec();
        cluster.append(0, value.clone()).await?;
        cluster.expect_value(0, value.clone(), TIMEOUT).await;
        cluster.expect_value(1, value.clone(), TIMEOUT).await;
        cluster.expect_value(2, value.clone(), TIMEOUT).await;
        Ok(())
    }

    #[tokio::test]
    async fn reconciliation() -> anyhow::Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0, TIMEOUT).await;
        cluster.block(1).await;
        let value = b"0".to_vec();
        cluster.append(0, value.clone()).await?;
        cluster.expect_value(0, value.clone(), TIMEOUT).await;
        cluster.expect_value(2, value.clone(), TIMEOUT).await;
        cluster.unblock().await;
        cluster.expect_value(1, value.clone(), TIMEOUT).await;
        Ok(())
    }

    #[tokio::test]
    async fn reconciliation_multiple_values() -> anyhow::Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let mut cluster = TestCluster::new(3);
        cluster.start().await;
        cluster.expect_leader(0, TIMEOUT).await;
        let value = b"0".to_vec();
        cluster.append(0, value.clone()).await?;
        cluster.expect_value(0, value.clone(), TIMEOUT).await;
        cluster.expect_value(1, value.clone(), TIMEOUT).await;
        cluster.expect_value(2, value.clone(), TIMEOUT).await;
        let value2 = b"1".to_vec();
        let value3 = b"2".to_vec();
        cluster.block(2).await;
        cluster.append(0, value2.clone()).await?;
        cluster.append(0, value3.clone()).await?;
        cluster.expect_value(0, value2.clone(), TIMEOUT).await;
        cluster.expect_value(0, value3.clone(), TIMEOUT).await;
        cluster.expect_value(1, value2.clone(), TIMEOUT).await;
        cluster.expect_value(1, value3.clone(), TIMEOUT).await;
        cluster.unblock().await;
        cluster.expect_value(2, value2.clone(), TIMEOUT).await;
        cluster.expect_value(2, value3.clone(), TIMEOUT).await;
        Ok(())
    }
}
