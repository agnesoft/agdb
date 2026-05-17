use crate::DbKind;
use crate::test_server::ADMIN;
use crate::test_server::TestServer;
use crate::test_server::audit_entries;
use crate::test_server::audit_file;
use crate::test_server::backup_audit_file;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_error::TestError;
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use std::path::Path;

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    let backup_audit = backup_audit_file(&server.data_dir, owner, db);
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;

    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let audit = audit_file(&server.data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner, db, queries).await?.1;
    assert_eq!(
        results,
        vec![QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId(1),
                from: DbId::default(),
                to: DbId::default(),
                values: vec![]
            }]
        }]
    );
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn backup_overwrite() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    let queries = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let results = server
        .api
        .admin_db_exec(owner, db, queries)
        .await
        .unwrap_err()
        .description;
    assert_eq!(results, "Alias 'root' not found");
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn backup_of_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    let queries = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let results = server.api.admin_db_exec(owner, db, queries).await?.1;
    assert_eq!(results[0].result, 1);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn rollback_of_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    let audit = audit_file(&server.data_dir, owner, db);
    let backup_audit = backup_audit_file(&server.data_dir, owner, db);
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.admin_db_exec_mut(owner, db, queries).await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 2);

    let status = server.api.admin_db_rollback(owner, db).await?;
    assert_eq!(status, 201);
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 2);

    let status = server.api.admin_db_rollback(owner, db).await?;
    assert_eq!(status, 201);
    assert_eq!(audit_entries(&audit)?, 2);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let results = server
        .api
        .admin_db_exec(owner, db, queries)
        .await
        .unwrap_err()
        .description;
    assert_eq!(results, "Alias 'root' not found");

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn restore_rollback_syncs_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;

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
    let audit_file = audit_file(&server.data_dir, owner, db);
    let backup_audit_file = backup_audit_file(&server.data_dir, owner, db);
    assert!(Path::new(&backup_audit_file).exists());
    assert_eq!(audit_entries(&backup_audit_file)?, 1);

    server
        .api
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;
    assert!(Path::new(&audit_file).exists());
    assert_eq!(audit_entries(&audit_file)?, 2);

    let audit = server.api.admin_db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 2);

    server.api.admin_db_rollback(owner, db).await?;
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
    let audit = server.api.admin_db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 1);
    assert_eq!(audit_entries(&audit_file)?, 1);
    assert_eq!(audit_entries(&backup_audit_file)?, 2);

    server.api.admin_db_rollback(owner, db).await?;
    let err = server
        .api
        .admin_db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await
        .unwrap_err()
        .description;
    assert_eq!(err, "Alias 'root' not found");
    let audit = server.api.admin_db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 2);
    assert_eq!(audit_entries(&audit_file)?, 2);
    assert_eq!(audit_entries(&backup_audit_file)?, 1);

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
    let audit = server.api.admin_db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 1);
    assert_eq!(audit_entries(&audit_file)?, 1);
    assert_eq!(audit_entries(&backup_audit_file)?, 1);

    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn in_memory() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    let status = server.api.admin_db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(Path::new(&server.data_dir).join(owner).join(db).exists());
    let status = server.api.admin_db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let status = server.api.admin_db_rollback(owner, db).await?;
    assert_eq!(status, 201);
    Ok(())
}

#[cfg_attr(feature = "api", agdb::test_def())]
pub async fn restore_no_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Mapped).await?;
    let status = server
        .api
        .admin_db_restore(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 404);
    let status = server
        .api
        .admin_db_rollback(owner, db)
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
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.admin_db_add(owner, db, DbKind::Memory).await?;
    server.api.user_login(owner, owner).await?;
    let status = server
        .api
        .admin_db_backup(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_db_restore(owner, db)
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_db_rollback(owner, db)
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
        .admin_db_backup("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_db_restore("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    let status = server
        .api
        .admin_db_rollback("owner", "db")
        .await
        .unwrap_err()
        .status;
    assert_eq!(status, 401);
    Ok(())
}

#[cfg(feature = "api")]
pub fn test_defs() -> Vec<agdb::type_def::Type> {
    vec![
        __backup_type_def(),
        __backup_overwrite_type_def(),
        __backup_of_backup_type_def(),
        __rollback_of_backup_type_def(),
        __restore_rollback_syncs_audit_type_def(),
        __in_memory_type_def(),
        __restore_no_backup_type_def(),
        __non_admin_type_def(),
        __no_token_type_def(),
    ]
}
