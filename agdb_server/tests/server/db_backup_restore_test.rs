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
use agdb::DbElement;
use agdb::DbId;
use agdb::QueryBuilder;
use agdb::QueryResult;
use agdb_api::DbKind;
use std::path::Path;

async fn backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let status = server.api.db_backup(owner, db).await?;
    assert_eq!(status, 201);
    assert!(
        Path::new(&server.data_dir)
            .join(owner)
            .join("backups")
            .join(format!("{db}.bak"))
            .exists()
    );
    let backup_audit = backup_audit_dir(&server.data_dir, owner, db);
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;

    let status = server.api.db_restore(owner, db).await?;
    assert_eq!(status, 201);
    let audit = audit_dir(&server.data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let results = server.api.db_exec(owner, db, queries).await?.1;
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

async fn rollback_of_backup() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;
    let queries = &[QueryBuilder::insert()
        .nodes()
        .aliases(["root"])
        .query()
        .into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    let status = server.api.db_backup(owner, db).await?;
    assert_eq!(status, 201);
    let audit = audit_dir(&server.data_dir, owner, db);
    let backup_audit = backup_audit_dir(&server.data_dir, owner, db);
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::remove().ids("root").query().into()];
    server.api.db_exec_mut(owner, db, queries).await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 2);

    let status = server.api.db_rollback(owner, db).await?;
    assert_eq!(status, 201);
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 2);

    let status = server.api.db_rollback(owner, db).await?;
    assert_eq!(status, 201);
    assert_eq!(audit_entries(&audit)?, 2);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let queries = &[QueryBuilder::select().ids("root").query().into()];
    let results = server
        .api
        .db_exec(owner, db, queries)
        .await
        .unwrap_err()
        .description;
    assert_eq!(results, "Alias 'root' not found");

    Ok(())
}

async fn restore_rollback_syncs_audit() -> Result<(), TestError> {
    let mut server = TestServer::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    server.api.user_login(ADMIN, ADMIN).await?;
    server.api.admin_user_add(owner, owner).await?;
    server.api.user_login(owner, owner).await?;
    server.api.db_add(owner, db, DbKind::Mapped).await?;

    server
        .api
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert()
                .nodes()
                .aliases(["root"])
                .query()
                .into()],
        )
        .await?;
    server.api.db_backup(owner, db).await?;
    let audit_dir = audit_dir(&server.data_dir, owner, db);
    let backup_audit_dir = backup_audit_dir(&server.data_dir, owner, db);
    assert!(Path::new(&backup_audit_dir).exists());
    assert_eq!(audit_entries(&backup_audit_dir)?, 1);

    server
        .api
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;
    assert!(Path::new(&audit_dir).exists());
    assert_eq!(audit_entries(&audit_dir)?, 2);

    let audit = server.api.db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 2);

    server.api.db_rollback(owner, db).await?;
    let results = server
        .api
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1;
    assert_eq!(results[0].result, 1);
    let audit = server.api.db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 1);
    assert_eq!(audit_entries(&audit_dir)?, 1);
    assert_eq!(audit_entries(&backup_audit_dir)?, 2);

    server.api.db_rollback(owner, db).await?;
    let err = server
        .api
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await
        .unwrap_err()
        .description;
    assert_eq!(err, "Alias 'root' not found");
    let audit = server.api.db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 2);
    assert_eq!(audit_entries(&audit_dir)?, 2);
    assert_eq!(audit_entries(&backup_audit_dir)?, 1);

    server.api.db_restore(owner, db).await?;
    let results = server
        .api
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1;
    assert_eq!(results[0].result, 1);
    let audit = server.api.db_audit(owner, db).await?.1;
    assert_eq!(audit.0.len(), 1);
    assert_eq!(audit_entries(&audit_dir)?, 1);
    assert_eq!(audit_entries(&backup_audit_dir)?, 1);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![backup, rollback_of_backup, restore_rollback_syncs_audit,];
