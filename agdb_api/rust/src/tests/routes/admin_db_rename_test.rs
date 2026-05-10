use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn rename() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server.api.admin_db_rename(owner, db, owner, db2).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn rename_with_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server.api.admin_db_backup(owner, db).await?;
    let status = server.api.admin_db_rename(owner, db, owner, db2).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(
        !Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    assert!(Path::new(&server.data_dir).join(owner).join(db2).exists());
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db2}.bak"))
            .exists()
    );
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn transfer() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let owner2 = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_user_add(owner2, owner2).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server.api.admin_db_rename(owner, db, owner2, db).await?;
    assert_eq!(status, 201);
    assert!(!Path::new(&server.data_dir).join(owner).join(db).exists());
    assert!(Path::new(&server.data_dir).join(owner2).join(db).exists());
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn user_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_rename(owner, db, "not_found", db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn invalid() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_rename(owner, db, owner, "a\0a")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 467);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn db_not_found() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    let status = server
        .api
        .admin_db_rename(owner, "db", owner, "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn target_self() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server.api.admin_db_rename(owner, db, owner, db).await?;
    assert_eq!(status, 201);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn target_exists() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let db2 = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    server.api.admin_db_add(owner, db2, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_rename(owner, db, owner, db2)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 465);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_rename(owner, "db", owner, "dbx")
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
        .admin_db_rename("owner", "db", "owner", "dbx")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __rename_type_def(),
        __rename_with_backup_type_def(),
        __transfer_type_def(),
        __user_not_found_type_def(),
        __invalid_type_def(),
        __db_not_found_type_def(),
        __target_self_type_def(),
        __target_exists_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
