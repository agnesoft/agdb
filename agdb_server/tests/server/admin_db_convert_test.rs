use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::audit_dir;
use crate::test_server::audit_entries;
use crate::test_server::backup_audit_dir;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::QueryBuilder;
use agdb_api::DbKind;
use std::path::Path;

async fn memory_to_mapped_preserves_backup_and_audit() -> Result<(), TestError> {
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
            &[QueryBuilder::insert()
                .nodes()
                .aliases(["root"])
                .query()
                .into()],
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

    server
        .api
        .admin_db_convert(owner, db, DbKind::Mapped)
        .await?;
    server.api.admin_db_restore(owner, db).await?;

    let results = server
        .api
        .admin_db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1;
    assert_eq!(results[0].result, 1);

    let audit = audit_dir(&server.data_dir, owner, db);
    let backup_audit = backup_audit_dir(&server.data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    Ok(())
}

async fn file_to_memory_preserves_backup_and_audit() -> Result<(), TestError> {
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
            &[QueryBuilder::insert()
                .nodes()
                .aliases(["root"])
                .query()
                .into()],
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

    server
        .api
        .admin_db_convert(owner, db, DbKind::Memory)
        .await?;
    server.api.admin_db_restore(owner, db).await?;

    let results = server
        .api
        .admin_db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1;
    assert_eq!(results[0].result, 1);

    let audit = audit_dir(&server.data_dir, owner, db);
    let backup_audit = backup_audit_dir(&server.data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    memory_to_mapped_preserves_backup_and_audit,
    file_to_memory_preserves_backup_and_audit,
];
