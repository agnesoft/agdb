use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;

#[tokio::test]
async fn convert() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbType::Memory).await?;
    let status = server.api.db_convert(owner, db, DbType::Mapped).await?;
    assert_eq!(status, 201);
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbType::Mapped);

    Ok(())
}
