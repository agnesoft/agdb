use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::audit_entries;
use crate::test_server::audit_file;
use crate::test_server::backup_audit_file;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::QueryBuilder;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn memory_to_mapped() -> Result<(), TestError> {
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn same_type() -> Result<(), TestError> {
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

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn memory_to_mapped_preserves_backup_and_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    server
        .api
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().aliases(["root"]).query().into()],
        )
        .await?;
    server.api.admin_db_backup(owner, db).await?;
    server
        .api
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;

    server.api.admin_db_convert(owner, db, DbKind::Mapped).await?;
    server.api.admin_db_restore(owner, db).await?;

    let results = server
        .api
        .admin_db_exec(owner, db, &[QueryBuilder::select().ids("root").query().into()])
        .await?
        .1;
    assert_eq!(results[0].result, 1);

    let audit = audit_file(&server.data_dir, owner, db);
    let backup_audit = backup_audit_file(&server.data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn file_to_memory_preserves_backup_and_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::File).await?;
    server
        .api
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().aliases(["root"]).query().into()],
        )
        .await?;
    server.api.admin_db_backup(owner, db).await?;
    server
        .api
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;

    server.api.admin_db_convert(owner, db, DbKind::Memory).await?;
    server.api.admin_db_restore(owner, db).await?;

    let results = server
        .api
        .admin_db_exec(owner, db, &[QueryBuilder::select().ids("root").query().into()])
        .await?
        .1;
    assert_eq!(results[0].result, 1);

    let audit = audit_file(&server.data_dir, owner, db);
    let backup_audit = backup_audit_file(&server.data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn non_admin() -> Result<(), TestError> {
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
        __memory_to_mapped_preserves_backup_and_audit_type_def(),
        __file_to_memory_preserves_backup_and_audit_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
