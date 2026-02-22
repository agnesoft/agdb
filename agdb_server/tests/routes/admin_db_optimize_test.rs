use crate::ADMIN;
use crate::TestServer;
use crate::next_db_name;
use crate::next_user_name;
use agdb::QueryBuilder;
use agdb_api::DbKind;

#[tokio::test]
async fn optimize() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let original_size = server
        .api
        .admin_db_list()
        .await?
        .1
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap()
        .size;
    let (status, db) = server.api.admin_db_optimize(owner, db).await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn shrink_to_fit() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert().nodes().count(100).query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let original_size = server
        .api
        .admin_db_list()
        .await?
        .1
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap()
        .size;
    let (status, db) = server
        .api
        .admin_db_optimize_shrink_to_fit(owner, db)
        .await?;
    assert_eq!(status, 200);
    assert!(db.size < original_size);
    Ok(())
}

#[tokio::test]
async fn db_not_found() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_optimize("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_optimize(owner, "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_optimize("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
