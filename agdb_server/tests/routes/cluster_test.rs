use crate::create_cluster;
use crate::next_db_name;
use crate::next_user_name;
use crate::reqwest_client;
use crate::wait_for_leader;
use crate::wait_for_ready;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb::Comparison;
use agdb::QueryBuilder;
use agdb_api::AgdbApi;
use agdb_api::DbResource;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use agdb_api::ReqwestClient;

#[tokio::test]
async fn rebalance() -> anyhow::Result<()> {
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
async fn status() {
    let server = TestServer::new().await.unwrap();
    let (code, status) = server.api.cluster_status().await.unwrap();

    assert_eq!(code, 200);
    assert_eq!(status.len(), 0);
}

#[tokio::test]
async fn admin_db_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    let db_list = client.admin_db_list().await?.1;
    let server_db = db_list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_eq!(server_db.db, *db);
    assert_eq!(server_db.owner, *owner);
    assert_eq!(server_db.db_type, DbType::Memory);
    Ok(())
}

#[tokio::test]
async fn admin_db_backup_restore() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client.admin_db_backup(owner, db).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 1);
    client.admin_db_restore(owner, db).await?;
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 0);
    Ok(())
}

#[tokio::test]
async fn admin_db_clear() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 1);
    client.admin_db_clear(owner, db, DbResource::All).await?;
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].elements[0]
        .values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 0);
    Ok(())
}

#[tokio::test]
async fn admin_db_convert() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client.admin_db_convert(owner, db, DbType::Mapped).await?;
    let db_list = client.admin_db_list().await?.1;
    let server_db = db_list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_eq!(server_db.db_type, DbType::Mapped);
    Ok(())
}

#[tokio::test]
async fn admin_db_copy() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client.admin_db_copy(owner, db, owner, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 2);
    Ok(())
}

#[tokio::test]
async fn admin_db_delete() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    let admin_token = client.token.clone();
    client.user_login(owner, owner).await?;
    let user_token = client.token.clone();
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.token = admin_token;
    client.admin_db_delete(owner, db).await?;
    client.token = user_token;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}

#[tokio::test]
async fn admin_db_exec() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .query()
                .into()],
        )
        .await?;
    client.user_login(owner, owner).await?;
    let result = client
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1[0]
        .result;
    assert_eq!(result, 1);
    Ok(())
}

#[tokio::test]
async fn admin_db_optimize() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(100).query().into()],
        )
        .await?;
    let original_size = client
        .admin_db_list()
        .await?
        .1
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap()
        .size;
    client.admin_db_optimize(owner, db).await?;
    let server_db = client.admin_db_optimize(owner, db).await?.1;
    assert!(server_db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn admin_db_remove() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    let admin_token = client.token.clone();
    client.user_login(owner, owner).await?;
    let user_token = client.token.clone();
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.token = admin_token;
    client.admin_db_remove(owner, db).await?;
    client.token = user_token;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}

#[tokio::test]
async fn admin_db_rename() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client.admin_db_rename(owner, db, owner, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].db, *db2);
    assert_eq!(dbs[0].owner, *owner);
    Ok(())
}

#[tokio::test]
async fn admin_db_user_add_remove() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_user_add(user, user).await?;
    client.admin_db_add(owner, db, DbType::Memory).await?;
    client
        .admin_db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    let users = client.admin_db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 2);
    client.admin_db_user_remove(owner, db, user).await?;
    let users = client.admin_db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 1);
    Ok(())
}

#[tokio::test]
async fn admin_user_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    let user = &next_user_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.username.as_str() == user);
    assert!(added_user.is_some());
    Ok(())
}

#[tokio::test]
async fn admin_user_change_password() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    let user = &next_user_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    client
        .admin_user_change_password(user, "password123")
        .await?;
    client.user_login(user, "password123").await?;
    Ok(())
}

#[tokio::test]
async fn admin_cluster_logout() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let token = {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_user_login(user, user).await?;
        client.token.clone()
    };

    {
        let leader = cluster.apis.get_mut(0).unwrap();
        leader.token = token;
        leader.user_status().await?;
    }

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.cluster_admin_user_logout(user).await?;
    }

    assert_eq!(cluster.apis[0].user_status().await.unwrap_err().status, 401);

    Ok(())
}

#[tokio::test]
async fn admin_user_delete() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let client = cluster.apis.get_mut(1).unwrap();
    let user = &next_user_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.username.as_str() == user);
    assert!(added_user.is_some());
    client.admin_user_delete(user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.username.as_str() == user);
    assert!(added_user.is_none());
    Ok(())
}

#[tokio::test]
async fn db_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    let db_list = client.db_list().await?.1;
    assert_eq!(db_list[0].db, *db);
    assert_eq!(db_list[0].owner, *owner);
    assert_eq!(db_list[0].db_type, DbType::Memory);
    Ok(())
}

#[tokio::test]
async fn db_backup() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client.db_backup(owner, db).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].elements[0].values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 1);
    client.db_restore(owner, db).await?;
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].elements[0].values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 0);
    Ok(())
}

#[tokio::test]
async fn db_clear() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].elements[0].values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 1);
    client.db_clear(owner, db, DbResource::All).await?;
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].elements[0].values[0]
        .value
        .to_u64()?;
    assert_eq!(node_count, 0);
    Ok(())
}

#[tokio::test]
async fn db_convert() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client.db_convert(owner, db, DbType::Mapped).await?;
    let db_list = client.db_list().await?.1;
    assert_eq!(db_list[0].db_type, DbType::Mapped);
    Ok(())
}

#[tokio::test]
async fn db_copy() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client.db_copy(owner, db, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 2);
    Ok(())
}

#[tokio::test]
async fn db_delete() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.db_delete(owner, db).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}

#[tokio::test]
async fn db_exec() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    let res = client
        .db_exec_mut(
            owner,
            db,
            &[
                QueryBuilder::insert()
                    .nodes()
                    .aliases("root")
                    .query()
                    .into(),
                QueryBuilder::insert()
                    .nodes()
                    .values([
                        [("desc", "hello").into()],
                        [("desc", "world").into()],
                        [("desc", "!").into()],
                    ])
                    .query()
                    .into(),
                QueryBuilder::insert()
                    .edges()
                    .from("root")
                    .to(":1")
                    .query()
                    .into(),
                QueryBuilder::select()
                    .search()
                    .from("root")
                    .where_()
                    .key("desc")
                    .value(Comparison::Contains("o".into()))
                    .query()
                    .into(),
            ],
        )
        .await?;
    assert_eq!(res.1[3].result, 2);
    client.user_login(owner, owner).await?;
    let result = client
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1[0]
        .result;
    assert_eq!(result, 1);
    Ok(())
}

#[tokio::test]
async fn db_optimize() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(100).query().into()],
        )
        .await?;
    let original_size = client.db_list().await?.1[0].size;
    client.db_optimize(owner, db).await?;
    let server_db = client.db_optimize(owner, db).await?.1;
    assert!(server_db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn db_remove() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.db_remove(owner, db).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}

#[tokio::test]
async fn db_rename() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client.db_rename(owner, db, db2).await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].db, *db2);
    assert_eq!(dbs[0].owner, *owner);
    Ok(())
}

#[tokio::test]
async fn db_user_add_remove() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_user_add(user, user).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client
        .db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    let users = client.db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 2);
    client.db_user_remove(owner, db, user).await?;
    let users = client.db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 1);
    Ok(())
}

#[tokio::test]
async fn user_change_password() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let user = &next_user_name();
    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_user_login(user, user).await?;
        client.user_change_password(user, "password123").await?;
    }
    cluster.apis[0].user_login(user, "password123").await?;
    Ok(())
}

#[tokio::test]
async fn user_login() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;

    let token = {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.token.clone()
    };

    let leader = cluster.apis.get_mut(0).unwrap();
    leader.token = token;
    leader.user_status().await?;

    Ok(())
}

#[tokio::test]
async fn cluster_logout() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let token = {
        let client = cluster.apis.get_mut(1).unwrap();
        client.user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_user_login(user, user).await?;
        client.token.clone()
    };

    {
        let leader = cluster.apis.get_mut(0).unwrap();
        leader.token = token;
        leader.user_status().await?;
    }

    {
        let client = cluster.apis.get_mut(1).unwrap();
        client.cluster_user_logout().await?;
    }

    assert_eq!(cluster.apis[0].user_status().await.unwrap_err().status, 401);
    assert_eq!(cluster.apis[1].user_status().await.unwrap_err().status, 401);

    Ok(())
}
