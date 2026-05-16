use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn memory_to_mapped() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server
        .api
        .admin_db_convert(owner, db, DbKind::Mapped)
        .await?;
    assert_eq!(status, 201);
    server.user_login(owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbKind::Mapped);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn same_type() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server
        .api
        .admin_db_convert(owner, db, DbKind::Memory)
        .await?;
    assert_eq!(status, 201);
    server.user_login(owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].db_type, DbKind::Memory);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let user = &next_user_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(user, user).await?;
    server.user_login(user).await?;
    let status = server
        .api
        .admin_db_convert(user, "db", DbKind::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn no_token() -> Result<(), TestError> {
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

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __memory_to_mapped_type_def(),
        __same_type_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
