use crate::TestServer;
use crate::ADMIN;
use agdb_api::DbType;

#[tokio::test]
async fn memory_to_mapped() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    let status = server
        .api
        .admin_db_convert(owner, db, DbType::Mapped)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(owner, owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbType::Mapped);

    Ok(())
}

#[tokio::test]
async fn same_type() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let owner = &server.next_user_name();
    let db = &server.next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbType::Memory).await?;
    let status = server
        .api
        .admin_db_convert(owner, db, DbType::Memory)
        .await?;
    assert_eq!(status, 201);
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbType::Memory);

    Ok(())
}

#[tokio::test]
async fn non_admin() -> anyhow::Result<()> {
    let mut server = TestServer::new().await?;
    let user = &server.next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .admin_db_convert(user, "db1", DbType::Mapped)
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
        .admin_db_convert("user", "db", DbType::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);

    Ok(())
}
