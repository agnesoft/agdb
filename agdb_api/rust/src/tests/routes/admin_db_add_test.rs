use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::File).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn add_same_name_with_previous_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    assert_eq!(status, 201);
    server.api.admin_db_backup(owner, db).await?;
    server.api.admin_db_delete(owner, db).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    assert_eq!(status, 201);
    server.user_login(owner).await?;
    let list = server.api.db_list().await?.1;
    assert_eq!(list[0].backup, 0);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn db_already_exists() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    let status = server.api.admin_db_add(owner, db, DbKind::File).await?;
    assert_eq!(status, 201);
    let status = server
        .api
        .admin_db_add(owner, db, DbKind::File)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    server.user_login(ADMIN).await?;
    let status = server
        .api
        .admin_db_add("owner", "db", DbKind::Mapped)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.user_login(ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.user_login(owner).await?;
    let status = server
        .api
        .admin_db_add(owner, db, DbKind::Mapped)
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
        .admin_db_add("owner", "db", DbKind::Memory)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __add_type_def(),
        __add_same_name_with_previous_backup_type_def(),
        __db_already_exists_type_def(),
        __user_not_found_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
