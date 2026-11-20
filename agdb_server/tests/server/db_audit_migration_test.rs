use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::TestServerImpl;
use crate::test_server::audit_dir;
use crate::test_server::audit_entries;
use crate::test_server::backup_audit_dir;
use crate::test_server::next_db_name;
use crate::test_server::reqwest_client;
use crate::test_server::wait_for_ready;
use agdb::QueryBuilder;
use agdb_api::AgdbApi;
use agdb_api::DbKind;
use agdb_api::QueryAudit;
use agdb_api::ReqwestClient;
use std::path::Path;

fn legacy_audit(timestamps: &[u64]) -> anyhow::Result<String> {
    let entries: Vec<QueryAudit> = timestamps
        .iter()
        .map(|timestamp| QueryAudit {
            timestamp: *timestamp,
            username: ADMIN.to_string(),
            query: QueryBuilder::insert().nodes().count(1).query().into(),
        })
        .collect();
    Ok(serde_json::to_string(&entries)?)
}

async fn legacy_audit_migrated_on_start() -> anyhow::Result<()> {
    let mut server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    let db = &next_db_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.db_add(ADMIN, db, DbKind::Mapped).await?;
    client
        .db_exec_mut(
            ADMIN,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    client.db_backup(ADMIN, db).await?;
    client.admin_shutdown().await?;
    server.wait().await?;
    let owner_dir = Path::new(&server.data_dir).join(ADMIN);
    let legacy = owner_dir.join("audit").join(format!("{db}.log"));
    let backup_legacy = owner_dir.join("backups").join(format!("{db}.log"));
    std::fs::write(&legacy, legacy_audit(&[1_577_836_800, 1_577_923_200])?)?;
    std::fs::write(&backup_legacy, legacy_audit(&[1_577_836_800])?)?;

    server.restart()?;
    wait_for_ready(&client).await?;

    client.user_login(ADMIN, ADMIN).await?;
    let audit = client.db_audit(ADMIN, db).await?.1.0;
    assert_eq!(
        audit
            .iter()
            .map(|a| a.timestamp)
            .take(2)
            .collect::<Vec<_>>(),
        vec![1_577_836_800, 1_577_923_200]
    );
    assert_eq!(audit.len(), 3);
    assert!(!legacy.exists());
    assert!(!backup_legacy.exists());
    let audit = audit_dir(&server.data_dir, ADMIN, db);
    let backup_audit = backup_audit_dir(&server.data_dir, ADMIN, db);
    assert!(Path::new(&audit).join("2020-01-01.log").exists());
    assert!(Path::new(&audit).join("2020-01-02.log").exists());
    assert_eq!(audit_entries(&backup_audit)?, 2);
    Ok(())
}

async fn legacy_backup_audit_migrated_on(rollback: bool) -> anyhow::Result<()> {
    let server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    let db = &next_db_name();
    client.user_login(ADMIN, ADMIN).await?;
    client.db_add(ADMIN, db, DbKind::Mapped).await?;
    client
        .db_exec_mut(
            ADMIN,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    client.db_backup(ADMIN, db).await?;
    let backup_legacy = Path::new(&server.data_dir)
        .join(ADMIN)
        .join("backups")
        .join(format!("{db}.log"));
    std::fs::write(&backup_legacy, legacy_audit(&[1_577_836_800])?)?;

    if rollback {
        client.db_rollback(ADMIN, db).await?;
    } else {
        client.db_restore(ADMIN, db).await?;
    }

    let audit = client.db_audit(ADMIN, db).await?.1.0;
    assert_eq!(audit.len(), 2);
    assert_eq!(audit[0].timestamp, 1_577_836_800);
    assert!(!backup_legacy.exists());
    Ok(())
}

async fn legacy_backup_audit_migrated_on_restore() -> anyhow::Result<()> {
    legacy_backup_audit_migrated_on(false).await
}

async fn legacy_backup_audit_migrated_on_rollback() -> anyhow::Result<()> {
    legacy_backup_audit_migrated_on(true).await
}

async fn delete_keeps_the_audit_of_a_db_named_like_its_legacy_file() -> anyhow::Result<()> {
    let server = TestServerImpl::new().await?;
    let mut client = AgdbApi::new(
        ReqwestClient::with_client(reqwest_client()),
        &server.address,
    );
    let db = &next_db_name();
    let other = &format!("{db}.log");
    client.user_login(ADMIN, ADMIN).await?;
    client.db_add(ADMIN, db, DbKind::Mapped).await?;
    client.db_add(ADMIN, other, DbKind::Mapped).await?;
    client
        .db_exec_mut(
            ADMIN,
            other,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;

    client.db_delete(ADMIN, db).await?;

    assert_eq!(client.db_audit(ADMIN, other).await?.1.0.len(), 1);
    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    legacy_audit_migrated_on_start,
    legacy_backup_audit_migrated_on_restore,
    legacy_backup_audit_migrated_on_rollback,
    delete_keeps_the_audit_of_a_db_named_like_its_legacy_file,
];
