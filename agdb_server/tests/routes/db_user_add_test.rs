use crate::next_db_name;
use crate::next_user_name;
use crate::TestCluster;
use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;
use agdb_api::DbUserRole;
use agdb_api::ServerDatabase;

#[tokio::test]
async fn add_db_user() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(
        list,
        vec![ServerDatabase {
            name: format!("{}/{}", owner, db),
            db_type: DbType::Mapped,
            role: DbUserRole::Write,
            size: 2656,
            backup: 0,
        }]
    );
    Ok(())
}

#[tokio::test]
async fn add_db_user_as_non_owner_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let other = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_user_add(other, other).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_add(owner, db, other, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(other, other).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(
        list,
        vec![ServerDatabase {
            name: format!("{}/{}", owner, db),
            db_type: DbType::Mapped,
            role: DbUserRole::Write,
            size: 2656,
            backup: 0,
        }]
    );
    Ok(())
}

#[tokio::test]
async fn change_user_role() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(user, user).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(
        list,
        vec![ServerDatabase {
            name: format!("{}/{}", owner, db),
            db_type: DbType::Mapped,
            role: DbUserRole::Read,
            size: 2656,
            backup: 0,
        }]
    );
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_user_add(owner, "db", user, DbUserRole::Write)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn user_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let status = server
        .api
        .db_user_add(owner, db, "user", DbUserRole::Write)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn change_owner_role() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_add(owner, db, owner, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let other = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_user_add(other, other).await?;
    server.api.admin_db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_add(owner, db, other, DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_user_add("owner", "db", "other", DbUserRole::Read)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn cluster_db_user_add() -> anyhow::Result<()> {
    let mut cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    let client = cluster.apis.get_mut(1).unwrap();
    client.user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_user_add(user, user).await?;
    client.cluster_login(owner, owner).await?;
    client.db_add(owner, db, DbType::Memory).await?;
    client
        .db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    let users = client.db_user_list(owner, db).await?.1;
    assert_eq!(users.len(), 2);
    Ok(())
}
