use crate::ServerResult;
use crate::action::ClusterAction;
use crate::config::Config;
use crate::raft::Log;
use crate::server_db::ServerDb;
use crate::server_error::ServerError;
use agdb::Comparison;
use agdb::Db;
use agdb::DbId;
use agdb::DbKeyOrder;
use agdb::QueryBuilder;
use agdb::StorageData;
use agdb::Transaction;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub(crate) struct ClusterLog(pub(crate) Arc<RwLock<Db>>);

const CLUSTER_LOG: &str = "cluster_log";
const EXECUTED: &str = "executed";
const COMMITTED: &str = "committed";
pub(crate) const CLUSTER_LOG_FILE: &str = "agdb_server.log";

pub(crate) async fn new(config: &Config) -> ServerResult<ClusterLog> {
    std::fs::create_dir_all(&config.data_dir)?;
    let file = format!("{}/{}", config.data_dir, CLUSTER_LOG_FILE);
    ClusterLog::new(&file).await
}

impl ClusterLog {
    async fn new(name: &str) -> ServerResult<Self> {
        let mut db = Db::new(name)?;

        db.transaction_mut(|t| -> ServerResult<()> {
            let indexes: Vec<String> = t.exec(QueryBuilder::select().indexes().query())?.elements
                [0]
            .values
            .iter()
            .map(|kv| kv.key.to_string())
            .collect();

            if !indexes.iter().any(|i| i == EXECUTED) {
                t.exec_mut(QueryBuilder::insert().index(EXECUTED).query())?;
            }

            if !indexes.iter().any(|i| i == COMMITTED) {
                t.exec_mut(QueryBuilder::insert().index(COMMITTED).query())?;
            }

            if t.exec(QueryBuilder::select().ids(CLUSTER_LOG).query())
                .is_err()
            {
                t.exec_mut(QueryBuilder::insert().nodes().aliases(CLUSTER_LOG).query())?;
            }

            Ok(())
        })?;

        Ok(Self(Arc::new(RwLock::new(db))))
    }

    pub(crate) async fn append_log(&self, log: &Log<ClusterAction>) -> ServerResult<DbId> {
        self.0.write().await.transaction_mut(
            |t: &mut agdb::TransactionMut<'_, agdb::FileStorageMemoryMapped>| {
                let log_id = t
                    .exec_mut(QueryBuilder::insert().element(log).query())?
                    .elements[0]
                    .id;
                t.exec_mut(
                    QueryBuilder::insert()
                        .values([[(COMMITTED, false).into(), (EXECUTED, false).into()]])
                        .ids(log_id)
                        .query(),
                )?;
                t.exec_mut(
                    QueryBuilder::insert()
                        .edges()
                        .from(CLUSTER_LOG)
                        .to(log_id)
                        .query(),
                )?;
                Ok(log_id)
            },
        )
    }

    pub(crate) async fn cluster_log(&self) -> ServerResult<(u64, u64, u64)> {
        self.0.write().await.transaction_mut(|t| {
            if let Some(e) = t
                .exec(
                    QueryBuilder::select()
                        .values(["index", "term"])
                        .search()
                        .depth_first()
                        .from(CLUSTER_LOG)
                        .limit(1)
                        .where_()
                        .neighbor()
                        .query(),
                )?
                .elements
                .first()
            {
                let commit = if let Some(c) = t
                    .exec(
                        QueryBuilder::select()
                            .values("index")
                            .search()
                            .depth_first()
                            .from(CLUSTER_LOG)
                            .limit(1)
                            .where_()
                            .neighbor()
                            .and()
                            .not()
                            .keys(COMMITTED)
                            .query(),
                    )?
                    .elements
                    .first()
                {
                    c.values[0].value.to_u64()?
                } else {
                    0
                };

                return Ok((
                    e.values[0].value.to_u64()?,
                    e.values[1].value.to_u64()?,
                    commit,
                ));
            }

            Ok((0, 0, 0))
        })
    }

    pub(crate) async fn log_committed(&self, log_id: DbId) -> ServerResult<()> {
        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::remove().values(COMMITTED).ids(log_id).query())?;
        Ok(())
    }

    pub(crate) async fn log_executed(&self, log_id: DbId) -> ServerResult<()> {
        self.0
            .write()
            .await
            .exec_mut(QueryBuilder::remove().values(EXECUTED).ids(log_id).query())?;
        Ok(())
    }

    pub(crate) async fn logs_unexecuted(
        &self,
        index: u64,
    ) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.logs_until(index, EXECUTED).await
    }

    pub(crate) async fn logs_uncommitted(
        &self,
        index: u64,
    ) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.logs_until(index, COMMITTED).await
    }

    async fn logs_until(&self, index: u64, label: &str) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.0.read().await.transaction(|t| {
            let mut log_ids: Vec<(u64, DbId)> = t
                .exec(
                    QueryBuilder::select()
                        .values("index")
                        .search()
                        .index(label)
                        .value(false)
                        .query(),
                )?
                .elements
                .into_iter()
                .filter_map(|e| {
                    let log_index = e.values[0].value.to_u64().unwrap_or_default();

                    if log_index <= index {
                        Some((log_index, e.id))
                    } else {
                        None
                    }
                })
                .collect();
            log_ids.sort_by_key(|l| l.0);
            logs(t, log_ids.into_iter().map(|l| l.1).collect())
        })
    }

    pub(crate) async fn prune(&self, up_to_index: u64) -> ServerResult<()> {
        self.0.write().await.transaction_mut(|t| {
            t.exec_mut(
                QueryBuilder::remove()
                    .search()
                    .depth_first()
                    .from(CLUSTER_LOG)
                    .where_()
                    .neighbor()
                    .and()
                    .key("index")
                    .value(Comparison::LessThanOrEqual(up_to_index.into()))
                    // Only prune entries that are fully committed and executed.
                    .and()
                    .not()
                    .keys(COMMITTED)
                    .and()
                    .not()
                    .keys(EXECUTED)
                    .query(),
            )?;
            Ok(())
        })
    }

    pub(crate) async fn remove_uncommitted_logs(&self, from_index: u64) -> ServerResult<()> {
        self.0.write().await.transaction_mut(|t| {
            let logs: Vec<DbId> = t
                .exec(
                    QueryBuilder::select()
                        .values("index")
                        .search()
                        .index(COMMITTED)
                        .value(false)
                        .query(),
                )?
                .elements
                .into_iter()
                .filter_map(|e| {
                    let index = e.values[0].value.to_u64().unwrap_or_default();

                    if index >= from_index {
                        Some(e.id)
                    } else {
                        None
                    }
                })
                .collect();

            t.exec_mut(QueryBuilder::remove().ids(logs).query())
        })?;

        Ok(())
    }

    pub(crate) async fn logs_since(
        &self,
        from_index: u64,
    ) -> ServerResult<Vec<Log<ClusterAction>>> {
        self.0.read().await.transaction(|t| {
            let log_ids = t
                .exec(
                    QueryBuilder::select()
                        .values("index")
                        .search()
                        .depth_first()
                        .from(CLUSTER_LOG)
                        .order_by(DbKeyOrder::Asc("index".into()))
                        .where_()
                        .neighbor()
                        .and()
                        .key("index")
                        .value(Comparison::GreaterThan(from_index.into()))
                        .query(),
                )?
                .ids();

            logs(t, log_ids)
        })
    }
}

fn logs<T: StorageData>(
    t: &Transaction<T>,
    log_ids: Vec<DbId>,
) -> ServerResult<Vec<Log<ClusterAction>>> {
    Ok(t.exec(
        QueryBuilder::select()
            .elements::<Log<ClusterAction>>()
            .ids(log_ids)
            .query(),
    )?
    .try_into()?)
}

pub(crate) async fn migrate_from_server_db(
    server_db: &ServerDb,
    cluster_log: &ClusterLog,
) -> ServerResult<()> {
    if server_db
        .db
        .read()
        .await
        .exec(QueryBuilder::select().ids(CLUSTER_LOG).query())
        .is_err()
    {
        return Ok(());
    }

    if cluster_log
        .0
        .read()
        .await
        .exec(
            QueryBuilder::select()
                .edge_count_from()
                .ids(CLUSTER_LOG)
                .query(),
        )?
        .elements[0]
        .values[0]
        .value
        .to_u64()?
        > 0
    {
        return Err(ServerError::from(
            "Cluster log already has logs, cannot migrate from the server db",
        ));
    }

    let mut log_ids = Vec::new();
    let logs = server_db
        .db
        .read()
        .await
        .exec(
            QueryBuilder::select()
                .search()
                .depth_first()
                .from(CLUSTER_LOG)
                .where_()
                .neighbor()
                .query(),
        )?
        .elements;

    let logs = logs
        .into_iter()
        .map(|e| {
            log_ids.push(e.id);
            e.values
        })
        .rev()
        .collect::<Vec<_>>();

    cluster_log.0.write().await.transaction_mut(|t| {
        let ids = t.exec_mut(QueryBuilder::insert().nodes().values(logs).query())?;
        t.exec_mut(
            QueryBuilder::insert()
                .edges()
                .from(CLUSTER_LOG)
                .to(ids)
                .query(),
        )
    })?;

    server_db.db.write().await.transaction_mut(|t| {
        t.exec_mut(QueryBuilder::remove().index(COMMITTED).query())?;
        t.exec_mut(QueryBuilder::remove().index(EXECUTED).query())?;
        t.exec_mut(QueryBuilder::remove().ids(log_ids).query())?;
        t.exec_mut(QueryBuilder::remove().ids(CLUSTER_LOG).query())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::user_add::UserAdd;
    use crate::password;
    use agdb_api::LogLevelFilter;
    use agdb_api::config_impl::ConfigImpl;
    use agdb_api::config_impl::DEFAULT_CLUSTER_MAX_LOG_ENTRIES;
    use agdb_api::config_impl::DEFAULT_LOG_BODY_LIMIT;
    use agdb_api::config_impl::DEFAULT_REQUEST_BODY_LIMIT;
    use agdb_api::config_impl::DEFAULT_TOKEN_EXPIRY_SECONDS;
    use std::path::PathBuf;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    struct TestDir {
        directory: PathBuf,
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }

    fn test_config(test_name: &str) -> (Config, TestDir) {
        password::init(None);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "agdb_cluster_log_{test_name}_{}_{}",
            std::process::id(),
            timestamp
        ));

        let config = Config::new(ConfigImpl {
            bind: ":::3000".to_string(),
            address: "http://localhost:3000".to_string(),
            basepath: String::new(),
            static_roots: Vec::new(),
            admin: "admin".to_string(),
            log_level: LogLevelFilter::Info,
            log_body_limit: DEFAULT_LOG_BODY_LIMIT,
            request_body_limit: DEFAULT_REQUEST_BODY_LIMIT,
            data_dir: format!(
                "{}{sep}",
                directory.to_string_lossy(),
                sep = std::path::MAIN_SEPARATOR
            ),
            pepper_path: String::new(),
            tls_certificate: String::new(),
            tls_key: String::new(),
            tls_root: String::new(),
            cluster_token: "cluster".to_string(),
            cluster_heartbeat_timeout_ms: 1000,
            cluster_term_timeout_ms: 3000,
            cluster_election_factor_ms: 1000,
            cluster: vec![],
            cluster_max_log_entries: DEFAULT_CLUSTER_MAX_LOG_ENTRIES,
            cluster_node_id: 0,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_secs(),
            token_expiry_seconds: DEFAULT_TOKEN_EXPIRY_SECONDS,

            pepper: None,
        });

        (config, TestDir { directory })
    }

    fn test_log(index: u64, term: u64, suffix: &str) -> Log<ClusterAction> {
        Log {
            db_id: None,
            index,
            term,
            data: ClusterAction::UserAdd(UserAdd {
                user: format!("user_{suffix}"),
                password: vec![index as u8],
                salt: vec![term as u8],
            }),
        }
    }

    async fn seed_legacy_server_logs(
        server_db: &ServerDb,
        logs: &[Log<ClusterAction>],
    ) -> ServerResult<()> {
        server_db
            .db
            .write()
            .await
            .transaction_mut(|t| -> ServerResult<()> {
                let indexes: Vec<String> =
                    t.exec(QueryBuilder::select().indexes().query())?.elements[0]
                        .values
                        .iter()
                        .map(|kv| kv.key.to_string())
                        .collect();

                if !indexes.iter().any(|index| index == EXECUTED) {
                    t.exec_mut(QueryBuilder::insert().index(EXECUTED).query())?;
                }

                if !indexes.iter().any(|index| index == COMMITTED) {
                    t.exec_mut(QueryBuilder::insert().index(COMMITTED).query())?;
                }

                if t.exec(QueryBuilder::select().ids(CLUSTER_LOG).query())
                    .is_err()
                {
                    t.exec_mut(QueryBuilder::insert().nodes().aliases(CLUSTER_LOG).query())?;
                }

                for log in logs {
                    let log_id = t
                        .exec_mut(QueryBuilder::insert().element(log).query())?
                        .elements[0]
                        .id;
                    t.exec_mut(
                        QueryBuilder::insert()
                            .values([[(COMMITTED, false).into(), (EXECUTED, false).into()]])
                            .ids(log_id)
                            .query(),
                    )?;
                    t.exec_mut(
                        QueryBuilder::insert()
                            .edges()
                            .from(CLUSTER_LOG)
                            .to(log_id)
                            .query(),
                    )?;
                }

                Ok(())
            })?;

        Ok(())
    }

    async fn legacy_server_log_exists(server_db: &ServerDb) -> bool {
        server_db
            .db
            .read()
            .await
            .exec(QueryBuilder::select().ids(CLUSTER_LOG).query())
            .is_ok()
    }

    #[tokio::test]
    async fn migrate_from_server_db_noop_when_nothing_to_migrate() -> ServerResult<()> {
        let (config, _directory) = test_config("noop");
        let server_db =
            crate::server_db::new(&config, tokio::sync::broadcast::channel(1).0.subscribe())
                .await?;
        let cluster_log = crate::cluster_log::new(&config).await?;

        migrate_from_server_db(&server_db, &cluster_log).await?;

        assert!(!legacy_server_log_exists(&server_db).await);
        assert!(cluster_log.logs_since(0).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn migrate_from_server_db_moves_logs_in_order() -> ServerResult<()> {
        let (config, _directory) = test_config("moves_logs");
        let server_db =
            crate::server_db::new(&config, tokio::sync::broadcast::channel(1).0.subscribe())
                .await?;
        let cluster_log = crate::cluster_log::new(&config).await?;
        let logs = vec![
            test_log(1, 1, "first"),
            test_log(2, 1, "second"),
            test_log(3, 2, "third"),
        ];

        seed_legacy_server_logs(&server_db, &logs).await?;
        crate::cluster_log::migrate_from_server_db(&server_db, &cluster_log).await?;

        assert!(!legacy_server_log_exists(&server_db).await);

        let migrated_logs = cluster_log.logs_since(0).await?;
        let migrated_indices = migrated_logs
            .iter()
            .map(|log| log.index)
            .collect::<Vec<_>>();
        let migrated_terms = migrated_logs.iter().map(|log| log.term).collect::<Vec<_>>();

        assert_eq!(migrated_indices, vec![1, 2, 3]);
        assert_eq!(migrated_terms, vec![1, 1, 2]);

        Ok(())
    }

    #[tokio::test]
    async fn migrate_from_server_db_errors_when_both_logs_exist() -> ServerResult<()> {
        let (config, _directory) = test_config("both_have_logs");
        let server_db =
            crate::server_db::new(&config, tokio::sync::broadcast::channel(1).0.subscribe())
                .await?;
        let cluster_log = crate::cluster_log::new(&config).await?;

        seed_legacy_server_logs(&server_db, &[test_log(1, 1, "server")]).await?;
        cluster_log.append_log(&test_log(2, 1, "cluster")).await?;

        let error = migrate_from_server_db(&server_db, &cluster_log)
            .await
            .expect_err("migration should fail when both logs exist");

        assert!(
            error
                .description
                .contains("Cluster log already has logs, cannot migrate from the server db")
        );

        Ok(())
    }

    #[tokio::test]
    async fn prune_removes_entries_up_to_index() -> ServerResult<()> {
        let (config, _directory) = test_config("prune_basic");
        let cluster_log = crate::cluster_log::new(&config).await?;

        for i in 1..=10 {
            cluster_log
                .append_log(&test_log(i, 1, &format!("entry_{i}")))
                .await?;
        }

        cluster_log.prune(5).await?;

        let remaining = cluster_log.logs_since(0).await?;
        let indices: Vec<u64> = remaining.iter().map(|l| l.index).collect();
        assert_eq!(indices, vec![6, 7, 8, 9, 10]);

        Ok(())
    }

    #[tokio::test]
    async fn prune_is_idempotent() -> ServerResult<()> {
        let (config, _directory) = test_config("prune_idempotent");
        let cluster_log = crate::cluster_log::new(&config).await?;

        for i in 1..=5 {
            cluster_log
                .append_log(&test_log(i, 1, &format!("entry_{i}")))
                .await?;
        }

        cluster_log.prune(3).await?;
        cluster_log.prune(3).await?; // second call is no-op

        let remaining = cluster_log.logs_since(0).await?;
        assert_eq!(remaining.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn logs_since_after_prune() -> ServerResult<()> {
        let (config, _directory) = test_config("logs_since_pruned");
        let cluster_log = crate::cluster_log::new(&config).await?;

        for i in 1..=10 {
            cluster_log
                .append_log(&test_log(i, 1, &format!("entry_{i}")))
                .await?;
        }

        cluster_log.prune(5).await?;

        // Ask for logs since index 3 — entries 4,5 are pruned, should return 6-10
        let logs = cluster_log.logs_since(3).await?;
        let indices: Vec<u64> = logs.iter().map(|l| l.index).collect();
        assert_eq!(indices, vec![6, 7, 8, 9, 10]);

        // Ask for logs since index 7 — should return 8,9,10
        let logs = cluster_log.logs_since(7).await?;
        let indices: Vec<u64> = logs.iter().map(|l| l.index).collect();
        assert_eq!(indices, vec![8, 9, 10]);

        Ok(())
    }

    #[tokio::test]
    async fn logs_since_sorted_ascending() -> ServerResult<()> {
        let (config, _directory) = test_config("logs_since_sorted");
        let cluster_log = crate::cluster_log::new(&config).await?;

        for i in 1..=5 {
            cluster_log
                .append_log(&test_log(i, 1, &format!("entry_{i}")))
                .await?;
        }

        let logs = cluster_log.logs_since(0).await?;
        let indices: Vec<u64> = logs.iter().map(|l| l.index).collect();
        assert_eq!(indices, vec![1, 2, 3, 4, 5]);

        Ok(())
    }

    #[tokio::test]
    async fn cluster_log_state_after_prune() -> ServerResult<()> {
        let (config, _directory) = test_config("state_after_prune");
        let cluster_log = crate::cluster_log::new(&config).await?;

        for i in 1..=10 {
            cluster_log
                .append_log(&test_log(i, 2, &format!("entry_{i}")))
                .await?;
        }

        cluster_log.prune(7).await?;

        // cluster_log() returns the state of the most recent entry
        let (index, term, _commit) = cluster_log.cluster_log().await?;
        assert_eq!(index, 10);
        assert_eq!(term, 2);

        Ok(())
    }

    #[tokio::test]
    async fn prune_all_entries() -> ServerResult<()> {
        let (config, _directory) = test_config("prune_all");
        let cluster_log = crate::cluster_log::new(&config).await?;

        for i in 1..=5 {
            cluster_log
                .append_log(&test_log(i, 1, &format!("entry_{i}")))
                .await?;
        }

        cluster_log.prune(5).await?;

        let logs = cluster_log.logs_since(0).await?;
        assert!(logs.is_empty());

        Ok(())
    }
}
