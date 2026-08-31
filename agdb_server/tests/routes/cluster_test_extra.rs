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

    for _ in 0..10 {
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
            assert_eq!(value, 9, "snapshot must transfer the post backup key value");
            break;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

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
