use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb_api::DbKind;

async fn memory_to_mapped() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server
        .api
        .admin_db_convert(owner, db, DbKind::Mapped)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(owner, owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbKind::Mapped);

    Ok(())
}

async fn same_type() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server
        .api
        .admin_db_convert(owner, db, DbKind::Memory)
        .await?;
    assert_eq!(status, 201);
    server.api.user_login(owner, owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbKind::Memory);

    Ok(())
}

async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.api.user_login(user, user).await?;
    let status = server
        .api
        .admin_db_convert(user, "db", DbKind::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

async fn no_token() -> Result<(), TestError> {
    let server = TestServer::new().await?;
    let status = server
        .api
        .admin_db_convert("owner", "db", DbKind::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![memory_to_mapped, same_type, non_admin, no_token,];
