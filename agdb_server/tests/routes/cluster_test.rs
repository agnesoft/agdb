use crate::create_cluster;
use crate::wait_for_leader;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::AgdbApi;
use agdb_api::DbResource;
use agdb_api::DbType;
use agdb_api::DbUser;
use agdb_api::DbUserRole;
use agdb_api::ReqwestClient;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;

#[tokio::test]
async fn rebalance() -> anyhow::Result<()> {
    let mut servers = create_cluster(3).await?;
    let mut leader = AgdbApi::new(ReqwestClient::new(), &servers[0].address);
    leader.user_login(ADMIN, ADMIN).await?;
    leader.admin_shutdown().await?;
    assert!(servers[0].process.wait()?.success());

    let mut statuses = Vec::with_capacity(servers.len());

    for server in &servers[1..] {
        let status = wait_for_leader(&AgdbApi::new(ReqwestClient::new(), &server.address)).await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    let dir = &servers[0].dir;
    servers[0].process = Command::cargo_bin("agdb_server")?
        .current_dir(dir)
        .spawn()?;

    statuses.clear();

    for server in &servers {
        let status = wait_for_leader(&AgdbApi::new(ReqwestClient::new(), &server.address)).await?;
        statuses.push(status);
    }

    for status in &statuses {
        assert_eq!(statuses[0], *status);
    }

    Ok(())
}

#[tokio::test]
async fn user() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.cluster_login(ADMIN, ADMIN).await?;
        client.admin_user_add("user1", "password123").await?;
        client.user_login("user1", "password123").await?;
    }

    {
        let leader = cluster.apis.get_mut(0).unwrap();
        leader.user_login(ADMIN, ADMIN).await?;
        leader.admin_cluster_logout("user1").await?;
        leader.admin_user_remove("user1").await?;
    }

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.user_status().await?;
        client.cluster_logout().await?;
        assert_eq!(client.user_status().await.unwrap_err().status, 401);
    }

    Ok(())
}

#[tokio::test]
async fn db() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;

    client.db_backup(ADMIN, "db1").await?;
    let db = &client.db_list().await?.1[0];
    assert_ne!(db.backup, 0);
    client.db_restore(ADMIN, "db1").await?;

    let db = client.db_clear(ADMIN, "db1", DbResource::Backup).await?.1;
    assert_eq!(db.backup, 0);

    client.db_convert(ADMIN, "db1", DbType::Mapped).await?;
    let db = &client.db_list().await?.1[0];
    assert_eq!(db.db_type, DbType::Mapped);

    client.db_copy(ADMIN, "db1", ADMIN, "db2").await?;
    let db = &client.db_list().await?.1[1];
    assert_eq!(db.name, "admin/db2");
    client.db_backup(ADMIN, "db2").await?;

    client.db_remove(ADMIN, "db2").await?;
    assert_eq!(client.db_list().await?.1.len(), 1);

    client.db_add(ADMIN, "db2", DbType::Memory).await?;
    let db = &client.db_list().await?.1[1];
    assert_eq!(db.name, "admin/db2");
    assert_ne!(db.backup, 0);

    client.db_delete(ADMIN, "db2").await?;
    assert_eq!(client.db_list().await?.1.len(), 1);

    client
        .db_exec(
            ADMIN,
            "db1",
            &[QueryBuilder::insert().nodes().count(100).query().into()],
        )
        .await?;
    let node_count = client
        .db_exec(
            ADMIN,
            "db1",
            &[QueryBuilder::select().node_count().query().into()],
        )
        .await?
        .1[0]
        .elements[0]
        .values[0]
        .value
        .to_u64()
        .unwrap();
    assert_eq!(node_count, 100);

    let orig_size = client.db_list().await?.1[0].size;
    let db_size = client.db_optimize(ADMIN, "db1").await?.1.size;
    assert!(db_size < orig_size);

    client.db_rename(ADMIN, "db1", ADMIN, "db2").await?;
    let db = &client.db_list().await?.1[0];
    assert_eq!(db.name, "admin/db2");

    client.admin_user_add("user2", "password123").await?;
    client
        .db_user_add(ADMIN, "db2", "user2", DbUserRole::Write)
        .await?;
    let users = client.db_user_list(ADMIN, "db2").await?.1;
    let expected = vec![
        DbUser {
            user: ADMIN.to_string(),
            role: DbUserRole::Admin,
        },
        DbUser {
            user: "user2".to_string(),
            role: DbUserRole::Write,
        },
    ];
    assert_eq!(users, expected);
    client.db_user_remove(ADMIN, "db2", "user2").await?;
    let users = client.db_user_list(ADMIN, "db2").await?.1;
    let expected = vec![DbUser {
        user: ADMIN.to_string(),
        role: DbUserRole::Admin,
    }];
    assert_eq!(users, expected);

    Ok(())
}

#[tokio::test]
async fn db_admin() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    client.cluster_login(ADMIN, ADMIN).await?;

    client.admin_db_add(ADMIN, "db1", DbType::Memory).await?;
    let db = &client.db_list().await?.1[0];
    assert_eq!(db.name, "admin/db1");
    assert_eq!(db.db_type, DbType::Memory);

    client.admin_db_backup(ADMIN, "db1").await?;
    let db = &client.db_list().await?.1[0];
    assert_ne!(db.backup, 0);
    client.admin_db_restore(ADMIN, "db1").await?;

    let db = client.db_clear(ADMIN, "db1", DbResource::Backup).await?.1;
    assert_eq!(db.backup, 0);

    client
        .admin_db_convert(ADMIN, "db1", DbType::Mapped)
        .await?;
    let db = &client.db_list().await?.1[0];
    assert_eq!(db.db_type, DbType::Mapped);

    client.admin_db_copy(ADMIN, "db1", ADMIN, "db2").await?;
    let db = &client.db_list().await?.1[1];
    assert_eq!(db.name, "admin/db2");
    client.admin_db_backup(ADMIN, "db2").await?;

    client.admin_db_remove(ADMIN, "db2").await?;
    assert_eq!(client.db_list().await?.1.len(), 1);

    client.admin_db_add(ADMIN, "db2", DbType::Memory).await?;
    let db = &client.db_list().await?.1[1];
    assert_eq!(db.name, "admin/db2");
    assert_ne!(db.backup, 0);

    client.admin_db_delete(ADMIN, "db2").await?;
    assert_eq!(client.db_list().await?.1.len(), 1);

    client
        .admin_db_exec(
            ADMIN,
            "db1",
            &[QueryBuilder::insert().nodes().count(100).query().into()],
        )
        .await?;
    let node_count = client
        .admin_db_exec(
            ADMIN,
            "db1",
            &[QueryBuilder::select().node_count().query().into()],
        )
        .await?
        .1[0]
        .elements[0]
        .values[0]
        .value
        .to_u64()
        .unwrap();
    assert_eq!(node_count, 100);

    let orig_size = client.admin_db_list().await?.1[0].size;
    let db_size = client.admin_db_optimize(ADMIN, "db1").await?.1.size;
    assert!(db_size < orig_size);

    client.admin_db_rename(ADMIN, "db1", ADMIN, "db2").await?;
    let db = &client.db_list().await?.1[0];
    assert_eq!(db.name, "admin/db2");

    client.admin_user_add("user2", "password123").await?;
    client
        .admin_db_user_add(ADMIN, "db2", "user2", DbUserRole::Write)
        .await?;
    let users = client.admin_db_user_list(ADMIN, "db2").await?.1;
    let expected = vec![
        DbUser {
            user: ADMIN.to_string(),
            role: DbUserRole::Admin,
        },
        DbUser {
            user: "user2".to_string(),
            role: DbUserRole::Write,
        },
    ];
    assert_eq!(users, expected);
    client.admin_db_user_remove(ADMIN, "db2", "user2").await?;
    let users = client.admin_db_user_list(ADMIN, "db2").await?.1;
    let expected = vec![DbUser {
        user: ADMIN.to_string(),
        role: DbUserRole::Admin,
    }];
    assert_eq!(users, expected);

    Ok(())
}

#[tokio::test]
async fn status() {
    let server = TestServer::new().await.unwrap();
    let (code, status) = server.api.cluster_status().await.unwrap();

    assert_eq!(code, 200);
    assert_eq!(status.len(), 0);
}
