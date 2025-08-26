use crate::ADMIN;
use crate::TestServer;
use crate::next_db_name;
use crate::next_user_name;
use agdb_api::DbKind;
use agdb_api::DbUserRole;

#[tokio::test]
async fn remove() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    let list = server.api.db_user_list(owner, db).await?.1;
    assert_eq!(list.len(), 2);
    let status = server.api.db_user_remove(owner, db, user).await?;
    assert_eq!(status, 204);
    let list = server.api.db_user_list(owner, db).await?.1;
    assert_eq!(list.len(), 1);
    Ok(())
}

#[tokio::test]
async fn remove_owner() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Admin)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_remove(owner, db, owner)
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
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Write)
        .await?;
    server
        .api
        .admin_db_user_add(owner, db, other, DbUserRole::Write)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .db_user_remove(owner, db, other)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
    Ok(())
}

#[tokio::test]
async fn remove_self() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let user = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server
        .api
        .admin_db_user_add(owner, db, user, DbUserRole::Read)
        .await?;
    server.api.user_login(user, user).await?;
    let status = server.api.db_user_remove(owner, db, user).await?;
    assert_eq!(status, 204);
    let list = server.api.db_list().await?.1;
    assert!(list.is_empty());
    Ok(())
}

#[tokio::test]
async fn remove_self_owner() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .db_user_remove(owner, db, owner)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 403);
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
        .db_user_remove(owner, "db", user)
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
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .db_user_remove(owner, db, "user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .db_user_remove("owner", "db", "user")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
