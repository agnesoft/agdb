use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbBackupPolicy;
use agdb_api::DbType;

#[tokio::test]
async fn clear() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Mapped).await?;
    let queries = &vec![QueryBuilder::insert().nodes().count(1).query().into()];
    server.api.db_exec(owner, db, queries).await?;
    let (status, db) = server
        .api
        .db_clear(owner, db, DbBackupPolicy::Ignore)
        .await?;
    assert_eq!(status, 200);
    assert_eq!(db.size, 0);
    Ok(())
}
