use agdb::QueryBuilder;
use agdb_api::AgdbApi;
use agdb_api::DbKind;
use agdb_api::ReqwestClient;
use agdb_api::test_server::ADMIN;
use agdb_api::test_server::reqwest_client;
use agdb_api::test_server::test_cluster::create_cluster;
use agdb_api::test_server::test_cluster::create_cluster_with_max_log_entries;
use agdb_api::test_server::test_cluster::wait_for_leader;
use agdb_api::test_server::test_error::TestError;
use agdb_api::test_server::wait_for_ready;

#[tokio::test]
async fn snapshot_transfer() -> Result<(), TestError> {
    let mut servers = create_cluster_with_max_log_entries(3, 1).await?;
    let mut follower = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );
    follower.cluster_user_login(ADMIN, ADMIN).await?;
    follower.admin_shutdown().await?;
    servers[2].wait().await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader
        .db_add(ADMIN, "snapshot_test", DbKind::Mapped)
        .await?;
    leader
        .db_exec_mut(
            ADMIN,
            "snapshot_test",
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .values(vec![vec![("key", 1).into()]])
                .query()
                .into()],
        )
        .await?;
    leader.db_backup(ADMIN, "snapshot_test").await?;

    for i in 0..10 {
        leader
            .db_exec_mut(
                ADMIN,
                "snapshot_test",
                &[QueryBuilder::insert()
                    .values(vec![vec![("key", i).into()]])
                    .ids("root")
                    .query()
                    .into()],
            )
            .await?;
    }

    // Restart the leader to flush queued messages for downed follower (required on Win)
    leader.admin_shutdown().await?;
    servers[0].wait().await?;
    servers[0].restart()?;
    wait_for_ready(&leader).await?;
    let node1 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    wait_for_leader(&node1).await?;

    servers[2].restart()?;
    wait_for_ready(&follower).await?;

    let mut synced = false;

    for _ in 0..3 {
        if let Ok(result) = follower
            .db_exec(
                ADMIN,
                "snapshot_test",
                &[QueryBuilder::select()
                    .values("key")
                    .ids("root")
                    .query()
                    .into()],
            )
            .await
            && let Ok(value) = result.1[0].elements[0].values[0].value.to_u64()
        {
            synced = true;
            assert_eq!(value, 9, "snapshot must transfer the post backup key value");
            break;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    assert!(synced, "follower did not sync post backup changes");

    follower.db_restore(ADMIN, "snapshot_test").await?;

    assert_eq!(
        follower
            .db_exec(
                ADMIN,
                "snapshot_test",
                &[QueryBuilder::select()
                    .values("key")
                    .ids("root")
                    .query()
                    .into()]
            )
            .await?
            .1[0]
            .elements[0]
            .values[0]
            .value
            .to_u64()
            .expect("failed to read value"),
        1
    );

    Ok(())
}

#[tokio::test]
async fn rebalance() -> Result<(), TestError> {
    let mut servers = create_cluster(3, false).await?;
    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_shutdown().await?;
    servers[0].wait().await?;

    let mut statuses = Vec::with_capacity(servers.len() - 1);

    for server in &servers[1..] {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        ))
        .await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    servers[0].restart()?;
    wait_for_ready(&leader).await?;

    statuses.clear();

    for server in &servers {
        let status = wait_for_leader(&AgdbApi::new(
            ReqwestClient::with_client(reqwest_client()),
            &server.address,
        ))
        .await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}

#[tokio::test]
async fn log_catchup_after_append_failures() -> Result<(), TestError> {
    // Default max_log_entries (1000): logs are NOT pruned.
    // When the follower is down, Append delivery fails repeatedly.
    // After APPEND_FAILURE_THRESHOLD consecutive failures the leader sets
    // force_resync, causing the next heartbeat to carry an elevated
    // prune_index (= leader's log_commit).  The follower sees
    // prune_index > its own log_commit → needs_resync →
    // resync_from_leader tries log catch-up first, which succeeds
    // because the entries are still available (no pruning).
    let mut servers = create_cluster(3, false).await?;
    let mut follower = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );
    follower.cluster_user_login(ADMIN, ADMIN).await?;
    follower.admin_shutdown().await?;
    servers[2].wait().await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader
        .db_add(ADMIN, "log_catchup_test", DbKind::Mapped)
        .await?;
    leader
        .db_exec_mut(
            ADMIN,
            "log_catchup_test",
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .values(vec![vec![("key", 1).into()]])
                .query()
                .into()],
        )
        .await?;

    // Each mutation triggers an Append to the downed follower which fails,
    // incrementing append_failures.  After 3 failures force_resync is set.
    for i in 0..10 {
        leader
            .db_exec_mut(
                ADMIN,
                "log_catchup_test",
                &[QueryBuilder::insert()
                    .values(vec![vec![("key", i).into()]])
                    .ids("root")
                    .query()
                    .into()],
            )
            .await?;
    }

    servers[2].restart()?;
    wait_for_ready(&follower).await?;

    let node1 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    wait_for_leader(&node1).await?;

    let mut synced = false;

    for _ in 0..10 {
        if follower.user_login(ADMIN, ADMIN).await.is_ok()
            && let Ok(result) = follower
                .db_exec(
                    ADMIN,
                    "log_catchup_test",
                    &[QueryBuilder::select()
                        .values("key")
                        .ids("root")
                        .query()
                        .into()],
                )
                .await
            && let Ok(value) = result.1[0].elements[0].values[0].value.to_u64()
            && value == 9
        {
            synced = true;
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert!(synced, "follower did not sync after append failures");

    Ok(())
}

fn cluster_log_entry_count(data_dir: &str) -> u64 {
    let log_path = format!("{data_dir}/agdb_server.log");
    let db = agdb::Db::new(&log_path).expect("failed to open cluster log");
    db.exec(
        QueryBuilder::select()
            .edge_count_from()
            .ids("cluster_log")
            .query(),
    )
    .expect("failed to query cluster log")
    .elements[0]
        .values[0]
        .value
        .to_u64()
        .expect("failed to read edge count")
}

#[tokio::test]
async fn log_compaction_bounds_log_size() -> Result<(), TestError> {
    let mut servers = create_cluster_with_max_log_entries(3, 5).await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;

    leader
        .db_add(ADMIN, "compaction_test", DbKind::Memory)
        .await?;

    for i in 0..20 {
        leader
            .db_exec_mut(
                ADMIN,
                "compaction_test",
                &[QueryBuilder::insert()
                    .nodes()
                    .values(vec![vec![("key", i).into()]])
                    .query()
                    .into()],
            )
            .await?;
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    leader.admin_shutdown().await?;
    servers[0].wait().await?;
    let mut api1 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    api1.user_login(ADMIN, ADMIN).await?;
    api1.admin_shutdown().await?;
    servers[1].wait().await?;
    let mut api2 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );
    api2.user_login(ADMIN, ADMIN).await?;
    api2.admin_shutdown().await?;
    servers[2].wait().await?;

    for server in &servers {
        let count = cluster_log_entry_count(&server.data_dir);
        assert_eq!(
            count, 1,
            "Log should have one (last) entry in a healthy cluster: {}",
            server.address
        );
    }

    Ok(())
}

#[tokio::test]
async fn lagging_node_catches_up_after_restart() -> Result<(), TestError> {
    let mut servers = create_cluster_with_max_log_entries(3, 50).await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;

    leader.db_add(ADMIN, "catchup_test", DbKind::Mapped).await?;

    for i in 0..3 {
        leader
            .db_exec_mut(
                ADMIN,
                "catchup_test",
                &[QueryBuilder::insert()
                    .nodes()
                    .values(vec![vec![("key", i).into()]])
                    .query()
                    .into()],
            )
            .await?;
    }

    let mut follower_api = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );
    follower_api.user_login(ADMIN, ADMIN).await?;
    follower_api.admin_shutdown().await?;
    servers[2].wait().await?;

    for i in 3..8 {
        leader
            .db_exec_mut(
                ADMIN,
                "catchup_test",
                &[QueryBuilder::insert()
                    .nodes()
                    .values(vec![vec![("key", i).into()]])
                    .query()
                    .into()],
            )
            .await?;
    }

    servers[2].restart()?;
    follower_api = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );

    wait_for_ready(&follower_api).await?;
    wait_for_leader(&follower_api).await?;

    follower_api.user_login(ADMIN, ADMIN).await?;
    let result = follower_api
        .db_exec(
            ADMIN,
            "catchup_test",
            &[QueryBuilder::select().node_count().query().into()],
        )
        .await?;

    assert_eq!(result.1[0].result, 8);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    leader.admin_shutdown().await?;
    servers[0].wait().await?;
    let mut api1 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    api1.user_login(ADMIN, ADMIN).await?;
    api1.admin_shutdown().await?;
    servers[1].wait().await?;
    follower_api.admin_shutdown().await?;
    servers[2].wait().await?;

    for server in &servers {
        let count = cluster_log_entry_count(&server.data_dir);
        assert_eq!(
            count, 1,
            "Log should have one (last) entry after catching up: {}",
            server.address
        );
    }

    Ok(())
}

#[tokio::test]
async fn too_far_behind_triggers_resync() -> Result<(), TestError> {
    let mut servers = create_cluster_with_max_log_entries(3, 5).await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;
    leader.db_add(ADMIN, "resync_test", DbKind::Mapped).await?;

    for i in 0..3 {
        leader
            .db_exec_mut(
                ADMIN,
                "resync_test",
                &[QueryBuilder::insert()
                    .nodes()
                    .values(vec![vec![("key", i).into()]])
                    .query()
                    .into()],
            )
            .await?;
    }

    let mut follower_api = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );
    follower_api.user_login(ADMIN, ADMIN).await?;
    follower_api.admin_shutdown().await?;
    servers[2].wait().await?;

    for i in 3..20 {
        leader
            .db_exec_mut(
                ADMIN,
                "resync_test",
                &[QueryBuilder::insert()
                    .nodes()
                    .values(vec![vec![("key", i).into()]])
                    .query()
                    .into()],
            )
            .await?;
    }

    servers[2].restart()?;
    follower_api = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[2].address,
    );
    wait_for_ready(&follower_api).await?;

    let mut node_count = 0u64;

    for _ in 0..20 {
        if follower_api.user_login(ADMIN, ADMIN).await.is_ok()
            && let Ok(result) = follower_api
                .db_exec(
                    ADMIN,
                    "resync_test",
                    &[QueryBuilder::select().node_count().query().into()],
                )
                .await
        {
            node_count = result.1[0].result;

            if node_count == 20 {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert_eq!(node_count, 20);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    leader.admin_shutdown().await?;
    servers[0].wait().await?;
    let mut api1 = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    api1.user_login(ADMIN, ADMIN).await?;
    api1.admin_shutdown().await?;
    servers[1].wait().await?;
    let _ = follower_api.user_login(ADMIN, ADMIN).await;
    let _ = follower_api.admin_shutdown().await;
    servers[2].wait().await?;

    for server in &servers {
        let count = cluster_log_entry_count(&server.data_dir);
        assert_eq!(
            count, 1,
            "Log should have one (last) entry after full resync: {} has {count}",
            server.address
        );
    }

    Ok(())
}

#[tokio::test]
async fn per_db_resync_after_file_corruption() -> Result<(), TestError> {
    let servers = create_cluster(3, false).await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;

    // Use DbKind::File — FileStorage uses seek()+read_exact() on a raw
    // file descriptor (no mmap).  Truncating the file on disk from
    // outside the process is therefore immediately visible: the next
    // read_exact() returns UnexpectedEof → DbError → exec failure.
    leader.db_add(ADMIN, "per_db_resync", DbKind::File).await?;

    // Write via the follower — leader-forwarding means the call only
    // returns once Raft has committed, so the follower already has the
    // data when it completes (no poll loop needed).
    let mut follower = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    follower.cluster_user_login(ADMIN, ADMIN).await?;
    follower
        .db_exec_mut(
            ADMIN,
            "per_db_resync",
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .values(vec![vec![("key", 1).into()]])
                .query()
                .into()],
        )
        .await?;

    // Truncate the follower's DB file and WAL to 0 bytes while the
    // server is still running.  FileStorage's read() does
    // seek(pos) + read_exact(buf) → the next storage access returns
    // UnexpectedEof → the Raft exec fails on this node.
    let db_path = format!("{}/admin/per_db_resync", servers[1].data_dir);
    let wal_path = format!("{}/admin/.per_db_resync", servers[1].data_dir);
    std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&db_path)
        .expect("truncate db file");
    // WAL may already be empty/cleared; ignore missing file.
    let _ = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&wal_path);

    // Trigger a new mutation that touches the truncated DB.
    // On the follower the storage read fails → LOG_FAILED →
    // OkReport → leader sends ResyncDbs → per-DB resync downloads
    // the correct DB from the leader — all on the live running node,
    // no restart needed.
    leader
        .db_exec_mut(
            ADMIN,
            "per_db_resync",
            &[QueryBuilder::insert()
                .values(vec![vec![("key", 2).into()]])
                .ids("root")
                .query()
                .into()],
        )
        .await?;

    // Wait for per-DB resync to restore the database on the follower.
    let mut final_value = 0u64;
    for _ in 0..30 {
        if let Ok(result) = follower
            .db_exec(
                ADMIN,
                "per_db_resync",
                &[QueryBuilder::select()
                    .values("key")
                    .ids("root")
                    .query()
                    .into()],
            )
            .await
            && let Ok(v) = result.1[0].elements[0].values[0].value.to_u64()
        {
            final_value = v;

            if v == 2 {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert_eq!(
        final_value, 2,
        "per-DB resync did not restore data to follower"
    );

    Ok(())
}

#[tokio::test]
async fn server_db_resync_after_corruption() -> Result<(), TestError> {
    let mut servers = create_cluster(3, false).await?;

    let mut leader = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[0].address,
    );
    leader.user_login(ADMIN, ADMIN).await?;

    // Create a user via the leader (replicates through Raft to all nodes).
    leader.admin_user_add("test_user", "password123").await?;

    // Ensure follower has the user (can log in as test_user).
    let mut follower = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );

    let mut user_exists = false;
    for _ in 0..10 {
        if follower
            .user_login("test_user", "password123")
            .await
            .is_ok()
        {
            user_exists = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    assert!(user_exists, "follower did not replicate test_user");

    // The server_db uses mmap (FileStorageMemoryMapped) so we cannot
    // corrupt it from outside the running process without risking
    // SIGBUS.  Instead: shut down the follower, delete its server_db,
    // restart it.  The fresh empty server_db means the follower is at
    // the same Raft commit but is missing all user data.  The NEXT
    // server_db action will fail on the live node → per-DB resync
    // restores it automatically, no second restart required.
    follower.user_login(ADMIN, ADMIN).await?;
    follower.admin_shutdown().await?;
    servers[1].wait().await?;

    let server_db_path = format!("{}/agdb_server.agdb", servers[1].data_dir);
    let server_db_wal = format!("{}/.agdb_server.agdb", servers[1].data_dir);
    std::fs::remove_file(&server_db_path).expect("remove server_db");
    let _ = std::fs::remove_file(&server_db_wal); // WAL may not exist

    servers[1].restart()?;
    follower = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &servers[1].address,
    );
    wait_for_ready(&follower).await?;
    wait_for_leader(&follower).await?;

    // Trigger a server_db–targeted action from the leader.
    // cluster_user_login replicates SaveUserToken via Raft.
    // The follower's empty server_db has no "test_user" → exec fails →
    // LOG_FAILED → OkReport → leader sends ResyncDbs for server_db
    // → per-DB resync downloads the correct server_db from the leader.
    leader
        .cluster_user_login("test_user", "password123")
        .await?;

    // Wait for server_db resync to complete — verify by logging in
    // as test_user on the follower (which requires the user to exist
    // in the follower's server_db).
    let mut user_synced = false;
    for _ in 0..30 {
        if follower
            .user_login("test_user", "password123")
            .await
            .is_ok()
        {
            user_synced = true;
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert!(
        user_synced,
        "server_db resync did not restore test_user to follower"
    );

    Ok(())
}
