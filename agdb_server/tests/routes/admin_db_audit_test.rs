use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbType;

#[tokio::test]
async fn admin_audit() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let mut queries = vec![
        QueryBuilder::insert()
            .nodes()
            .aliases("root")
            .values([[("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids(":0").query().into(),
    ];
    server.api.db_exec(owner, db, &queries).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, results) = server.api.admin_db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0[0].user, owner.to_string());
    assert_eq!(results.0[0].query, queries.remove(0));
    Ok(())
}

#[tokio::test]
async fn admin_audit_db_empty() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    server.api.user_login(ADMIN, ADMIN).await?;
    let (status, results) = server.api.admin_db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0, vec![]);
    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_audit("owner_x", "db_x")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn audit_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_audit("owner_y", "db_y")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}
