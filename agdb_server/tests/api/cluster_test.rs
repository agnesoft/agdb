use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::api_with_agent;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_cluster::TestCluster;
use crate::test_server::test_error::TestError;
use agdb::Comparison;
use agdb::QueryBuilder;
use agdb::QueryType;
use agdb_api::DbKind;
use agdb_api::DbResource;
use agdb_api::DbUserRole;

async fn status() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let (code, statuses) = server.api.cluster_status().await?;

    assert_eq!(code, 200);
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].address, server.api.address());
    assert!(statuses[0].status);
    assert!(statuses[0].leader);

    Ok(())
}

async fn admin_db_add() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;

    let db_list = client.admin_db_list().await?.1;
    let server_db = db_list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_eq!(server_db.db, *db);
    assert_eq!(server_db.owner, *owner);
    assert_eq!(server_db.db_type, DbKind::Memory);
    Ok(())
}

async fn admin_db_clear() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query: &[QueryType] = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);
    client.admin_db_clear(owner, db, DbResource::All).await?;

    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;

    assert_eq!(node_count, 0);
    Ok(())
}

async fn admin_db_copy() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
    client.admin_db_copy(owner, db, owner, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 2);
    Ok(())
}

async fn admin_db_delete() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
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

async fn admin_db_exec() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
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

async fn admin_db_optimize() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[
                QueryBuilder::insert().nodes().count(100).query().into(),
                QueryBuilder::remove()
                    .search()
                    .elements()
                    .offset(50)
                    .query()
                    .into(),
            ],
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

async fn admin_db_remove() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
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

async fn admin_db_rename() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
    client.admin_db_rename(owner, db, owner, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].db, *db2);
    assert_eq!(dbs[0].owner, *owner);
    Ok(())
}

async fn admin_db_user_add_remove() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_user_add(user, user).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
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

async fn admin_user_add() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let mut client = cluster.follower();
    let user = &next_user_name();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    let users = client.admin_user_list().await?.1;
    let added_user = users.iter().find(|u| u.username.as_str() == user);
    assert!(added_user.is_some());
    Ok(())
}

async fn admin_user_change_password() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let mut client = cluster.follower();
    let user = &next_user_name();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    client
        .admin_user_change_password(user, "password123")
        .await?;
    client.user_login(user, "password123").await?;
    Ok(())
}

async fn admin_cluster_logout() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let mut follower = cluster.follower();
    follower.cluster_user_login(ADMIN, ADMIN).await?;
    follower.admin_user_add(user, user).await?;
    let admin_token = follower.token.clone();
    follower.user_login(user, user).await?;
    let user_token = follower.token.clone();
    follower.user_status().await?;
    follower.token = admin_token;
    follower.cluster_admin_user_logout(user).await?;
    follower.token = user_token;
    assert_eq!(follower.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn admin_user_delete() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let mut client = cluster.follower();
    let user = &next_user_name();
    client.cluster_user_login(ADMIN, ADMIN).await?;
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

async fn db_add() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    let db_list = client.db_list().await?.1;
    assert_eq!(db_list[0].db, *db);
    assert_eq!(db_list[0].owner, *owner);
    assert_eq!(db_list[0].db_type, DbKind::Memory);
    Ok(())
}

async fn db_clear() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let node_count_query: &[QueryType] = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);
    client.db_clear(owner, db, DbResource::All).await?;
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 0);
    Ok(())
}

async fn db_copy() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    client.db_copy(owner, db, db2).await?;
    client.user_login(owner, owner).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 2);
    Ok(())
}

async fn db_delete() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.db_delete(owner, db).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}

async fn db_exec() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
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

async fn db_optimize() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[
                QueryBuilder::insert().nodes().count(100).query().into(),
                QueryBuilder::remove()
                    .search()
                    .elements()
                    .offset(50)
                    .query()
                    .into(),
            ],
        )
        .await?;
    let original_size = client.db_list().await?.1[0].size;
    client.db_optimize(owner, db).await?;
    let server_db = client.db_optimize(owner, db).await?.1;
    assert!(server_db.size < original_size);
    Ok(())
}

async fn db_remove() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 1);
    client.db_remove(owner, db).await?;
    let dbs = client.db_list().await?.1.len();
    assert_eq!(dbs, 0);
    Ok(())
}

async fn db_rename() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
    client.db_rename(owner, db, db2).await?;
    let dbs = client.db_list().await?.1;
    assert_eq!(dbs.len(), 1);
    assert_eq!(dbs[0].db, *db2);
    assert_eq!(dbs[0].owner, *owner);
    Ok(())
}

async fn db_user_add_remove() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_user_add(user, user).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;
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

async fn user_change_password() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();
    {
        let mut client = cluster.follower();
        client.cluster_user_login(ADMIN, ADMIN).await?;
        client.admin_user_add(user, user).await?;
        client.cluster_user_login(user, user).await?;
        client.user_change_password(user, "password123").await?;
    }
    let mut leader = cluster.leader();
    leader.user_login(user, "password123").await?;
    Ok(())
}

async fn user_login() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;

    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    let token = client.token.clone();

    let mut leader = cluster.leader();
    leader.token = token;
    leader.user_status().await?;

    Ok(())
}

async fn cluster_logout() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let mut login = cluster.follower();
    login.cluster_user_login(ADMIN, ADMIN).await?;
    login.admin_user_add(user, user).await?;
    login.cluster_user_login(user, user).await?;
    let token = login.token.clone();

    let mut leader = cluster.leader();

    leader.token = token.clone();
    leader.user_status().await?;

    let mut client = cluster.follower();
    client.token = token.clone();
    client.cluster_user_logout().await?;

    assert_eq!(leader.user_status().await.unwrap_err().status, 401);
    assert_eq!(client.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn cluster_logout_all() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;

    client.cluster_user_login(user, user).await?;
    assert!(client.user_status().await?.1.login);
    let token1 = client.token.clone();

    client.cluster_user_login(user, user).await?;
    assert!(client.user_status().await?.1.login);
    let token2 = client.token.clone();

    client.cluster_user_logout_all().await?;

    let mut client = cluster.leader();
    client.token = token1;
    assert_eq!(client.user_status().await.unwrap_err().status, 401);

    client.token = token2;
    assert_eq!(client.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn cluster_logout_all_keep_self() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let mut admin = cluster.follower();
    admin.cluster_user_login(ADMIN, ADMIN).await?;
    admin.admin_user_add(user, user).await?;

    let address = admin.address().to_string();
    let mut client1 = api_with_agent(&address, "cluster-keep-self");
    let mut client2 = api_with_agent(&address, "cluster-keep-other");

    client1.cluster_user_login(user, user).await?;
    client2.cluster_user_login(user, user).await?;

    client1.cluster_user_logout_others().await?;

    assert!(client1.user_status().await?.1.login);
    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn cluster_logout_selected_session() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let mut admin = cluster.follower();
    admin.cluster_user_login(ADMIN, ADMIN).await?;
    admin.admin_user_add(user, user).await?;

    let address = admin.address().to_string();
    let mut client1 = api_with_agent(&address, "cluster-session-owner");
    let mut client2 = api_with_agent(&address, "cluster-session-revoked");

    client1.cluster_user_login(user, user).await?;
    client2.cluster_user_login(user, user).await?;

    let session_id = client1
        .user_status()
        .await?
        .1
        .sessions
        .iter()
        .find(|s| s.agent == "cluster-session-revoked")
        .map(|s| s.session.clone())
        .expect("expected session for revoked client");

    client1.cluster_user_logout_session(&session_id).await?;

    assert!(client1.user_status().await?.1.login);
    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn admin_cluster_logout_selected_session() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();

    let mut admin = cluster.follower();
    admin.cluster_user_login(ADMIN, ADMIN).await?;
    admin.admin_user_add(user, user).await?;

    let address = admin.address().to_string();
    let mut client1 = api_with_agent(&address, "cluster-admin-target-1");
    let mut client2 = api_with_agent(&address, "cluster-admin-target-2");

    client1.cluster_user_login(user, user).await?;
    client2.cluster_user_login(user, user).await?;

    let session_id = client2
        .user_status()
        .await?
        .1
        .sessions
        .into_iter()
        .find(|s| s.agent == "cluster-admin-target-2")
        .map(|s| s.session.clone())
        .expect("expected session for revoked client");

    admin
        .cluster_admin_user_logout_session(user, &session_id)
        .await?;

    assert!(client1.user_status().await?.1.login);
    assert_eq!(client2.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn dashboard() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let user = &next_user_name();
    let other = &next_user_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(user, user).await?;
    client.admin_user_add(other, other).await?;
    client.cluster_user_login(user, user).await?;
    client.dashboard_add(user, "ops", "{}").await?;
    client.dashboard_save(user, "ops", "{\"v\":2}").await?;
    assert_eq!(
        client.dashboard_get(user, "ops").await?.1.config,
        "{\"v\":2}"
    );
    client.dashboard_rename(user, "ops", "sales").await?;
    client
        .dashboard_user_add(user, "sales", other, DbUserRole::Read)
        .await?;
    client.user_home_dashboard_set(user, "sales").await?;
    let mut leader = cluster.leader();
    leader.user_login(user, user).await?;
    assert_eq!(
        leader.dashboard_get(user, "sales").await?.1.config,
        "{\"v\":2}"
    );
    assert_eq!(
        leader.user_home_dashboard().await?.1.map(|d| d.name),
        Some("sales".to_string())
    );
    assert_eq!(client.dashboard_user_list(user, "sales").await?.1.len(), 2);
    let names: Vec<String> = client
        .dashboard_list()
        .await?
        .1
        .into_iter()
        .map(|d| d.name)
        .collect();
    assert_eq!(names, vec!["sales".to_string()]);
    // A forwarded write is executed on the node it was sent to before it
    // answers, so that node reads it back at once.
    client.dashboard_public(user, "sales", true).await?;
    assert!(client.dashboard_get(user, "sales").await?.1.public);
    client.dashboard_public(user, "sales", false).await?;
    assert!(!client.dashboard_get(user, "sales").await?.1.public);
    client.user_home_dashboard_clear().await?;
    client.dashboard_user_remove(user, "sales", other).await?;
    assert_eq!(client.dashboard_user_list(user, "sales").await?.1.len(), 1);
    client.dashboard_remove(user, "sales").await?;
    assert!(client.dashboard_list().await?.1.is_empty());
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    status,
    admin_db_add,
    admin_db_clear,
    admin_db_copy,
    admin_db_delete,
    admin_db_exec,
    admin_db_optimize,
    admin_db_remove,
    admin_db_rename,
    admin_db_user_add_remove,
    admin_user_add,
    admin_user_change_password,
    admin_cluster_logout,
    admin_user_delete,
    db_add,
    db_clear,
    db_copy,
    db_delete,
    db_exec,
    db_optimize,
    db_remove,
    db_rename,
    db_user_add_remove,
    user_change_password,
    user_login,
    cluster_logout,
    cluster_logout_all,
    cluster_logout_all_keep_self,
    cluster_logout_selected_session,
    admin_cluster_logout_selected_session,
    dashboard,
];
