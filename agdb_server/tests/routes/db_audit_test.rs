use crate::next_db_name;
use crate::next_user_name;
use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbType;
use std::path::Path;

#[tokio::test]
async fn audit() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
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
    let (status, results) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0[0].user, owner.to_string());
    assert_eq!(results.0[0].query, queries.remove(0));
    Ok(())
}

#[tokio::test]
async fn audit_delete_db() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = vec![QueryBuilder::insert()
        .nodes()
        .aliases("root")
        .values([[("key", 1.1).into()]])
        .query()
        .into()];
    server.api.db_exec(owner, db, &queries).await?;
    let db_audit_file = Path::new(&server.data_dir)
        .join(owner)
        .join("audit")
        .join(format!("{}.log", db));
    assert!(db_audit_file.exists());
    let status = server.api.db_delete(owner, db).await?;
    assert_eq!(status, 204);
    assert!(!db_audit_file.exists());
    Ok(())
}

#[tokio::test]
async fn audit_db_empty() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let (status, results) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(results.0, vec![]);
    Ok(())
}

#[tokio::test]
async fn audit_no_token() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let status = server.api.db_audit("owner", "db").await.unwrap_err().status;
    assert_eq!(status, 401);
    Ok(())
}

#[tokio::test]
async fn repeated_query_with_db_audit() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    server
        .api
        .db_exec(
            owner,
            db,
            &vec![QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .query()
                .into()],
        )
        .await?;
    let (status, audit) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(audit.0.len(), 1);
    server
        .api
        .db_exec(
            owner,
            db,
            &vec![QueryBuilder::insert()
                .nodes()
                .aliases("root")
                .query()
                .into()],
        )
        .await?;
    let (status, audit2) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    assert_eq!(audit2.0.len(), 2);
    Ok(())
}
