use crate::TestServer;
use crate::ADMIN;
use agdb::QueryBuilder;
use agdb_api::DbType;
use agdb_api::QueryAudit;

#[tokio::test]
async fn read_write() -> anyhow::Result<()> {
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
            .values(vec![vec![("key", 1.1).into()]])
            .query()
            .into(),
        QueryBuilder::select().ids("root").query().into(),
    ];
    server.api.db_exec(owner, db, &queries).await?;
    let (status, results) = server.api.db_audit(owner, db).await?;
    assert_eq!(status, 200);
    let expected = vec![QueryAudit {
        timestamp: 0,
        user: owner.to_string(),
        query: queries.remove(0),
    }];
    assert_eq!(results.0, expected);
    Ok(())
}
