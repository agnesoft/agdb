use crate::runner::Test;
use crate::runner::tests;
use crate::test_server::ADMIN;
use crate::test_server::audit_dir;
use crate::test_server::audit_entries;
use crate::test_server::backup_audit_dir;
use crate::test_server::next_db_name;
use crate::test_server::next_user_name;
use crate::test_server::test_cluster::TestCluster;
use crate::test_server::test_error::TestError;
use crate::test_server::write_audit_entry;
use agdb::QueryBuilder;
use agdb_api::DbKind;
use std::path::Path;

async fn admin_db_backup_restore() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;

    let data_dir = cluster.data_dir(client.address());
    let audit = audit_dir(&data_dir, owner, db);
    let backup_audit = backup_audit_dir(&data_dir, owner, db);

    client.admin_db_backup(owner, db).await?;
    assert!(!Path::new(&audit).exists());
    assert!(!Path::new(&backup_audit).exists());

    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);

    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);

    client.admin_db_restore(owner, db).await?;
    assert!(!Path::new(&backup_audit).exists());
    assert!(!Path::new(&audit).exists());

    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;

    assert_eq!(node_count, 0);
    Ok(())
}

async fn admin_db_rollback() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Mapped).await?;

    let data_dir = cluster.data_dir(client.address());
    let audit = audit_dir(&data_dir, owner, db);
    let backup_audit = backup_audit_dir(&data_dir, owner, db);

    client.admin_db_backup(owner, db).await?;
    assert!(!Path::new(&audit).exists());
    assert!(!Path::new(&backup_audit).exists());

    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);

    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);

    client.admin_db_rollback(owner, db).await?;
    assert!(!Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 0);

    client.admin_db_rollback(owner, db).await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert!(!Path::new(&backup_audit).exists());

    let node_count = client.admin_db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);

    Ok(())
}

async fn admin_db_audit_trim() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    let audits = [
        audit_dir(&cluster.data_dir(client.address()), owner, db),
        audit_dir(&cluster.data_dir(cluster.leader().address()), owner, db),
    ];
    for audit in &audits {
        write_audit_entry(audit, "2020-01-01", 1_577_836_800)?;
    }

    client.admin_db_audit_trim(owner, db, 1).await?;

    for audit in &audits {
        assert!(!Path::new(audit).join("2020-01-01.log").exists());
        assert_eq!(audit_entries(audit)?, 1);
    }
    Ok(())
}

async fn admin_db_convert() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.admin_db_add(owner, db, DbKind::Memory).await?;

    client
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
    client.admin_db_backup(owner, db).await?;
    client
        .admin_db_exec_mut(
            owner,
            db,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;

    client.admin_db_convert(owner, db, DbKind::Mapped).await?;
    let db_list = client.admin_db_list().await?.1;
    let server_db = db_list
        .iter()
        .find(|d| d.db == *db && d.owner == *owner)
        .unwrap();
    assert_eq!(server_db.db_type, DbKind::Mapped);

    client.admin_db_restore(owner, db).await?;
    let results = client
        .admin_db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1;
    assert_eq!(results[0].result, 1);

    let data_dir = cluster.data_dir(client.address());
    let audit = audit_dir(&data_dir, owner, db);
    let backup_audit = backup_audit_dir(&data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    Ok(())
}

async fn admin_cluster_logout_all() -> Result<(), TestError> {
    let cluster = TestCluster::new_private().await?;
    let user1 = &next_user_name();
    let user2 = &next_user_name();

    let mut follower = cluster.follower();
    follower.cluster_user_login(ADMIN, ADMIN).await?;
    follower.admin_user_add(user1, user1).await?;
    follower.admin_user_add(user2, user2).await?;

    let mut client = cluster.follower();
    client.user_login(user1, user1).await?;
    client.user_status().await?;

    let mut leader = cluster.leader();
    leader.user_login(user2, user2).await?;
    leader.user_status().await?;

    follower.cluster_admin_user_logout_all().await?;

    assert_eq!(client.user_status().await.unwrap_err().status, 401);
    assert_eq!(leader.user_status().await.unwrap_err().status, 401);

    Ok(())
}

async fn db_backup() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;

    let data_dir = cluster.data_dir(client.address());
    let audit = audit_dir(&data_dir, owner, db);
    let backup_audit = backup_audit_dir(&data_dir, owner, db);

    client.db_backup(owner, db).await?;
    assert!(!Path::new(&audit).exists());
    assert!(!Path::new(&backup_audit).exists());

    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);

    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);

    client.db_restore(owner, db).await?;
    assert!(!Path::new(&backup_audit).exists());
    assert!(!Path::new(&audit).exists());

    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 0);
    Ok(())
}

async fn db_rollback() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Mapped).await?;

    let data_dir = cluster.data_dir(client.address());
    let audit = audit_dir(&data_dir, owner, db);
    let backup_audit = backup_audit_dir(&data_dir, owner, db);

    client.db_backup(owner, db).await?;
    assert!(!Path::new(&audit).exists());
    assert!(!Path::new(&backup_audit).exists());

    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::insert().nodes().count(1).query().into()],
        )
        .await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);

    let node_count_query = &[QueryBuilder::select().node_count().query().into()];
    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);

    client.db_rollback(owner, db).await?;
    assert!(!Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&backup_audit)?, 1);

    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 0);

    client.db_rollback(owner, db).await?;
    assert!(Path::new(&audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert!(!Path::new(&backup_audit).exists());

    let node_count = client.db_exec(owner, db, node_count_query).await?.1[0].result;
    assert_eq!(node_count, 1);

    Ok(())
}

async fn db_convert() -> Result<(), TestError> {
    let cluster = TestCluster::new().await?;
    let owner = &next_user_name();
    let db = &next_db_name();
    let mut client = cluster.follower();
    client.cluster_user_login(ADMIN, ADMIN).await?;
    client.admin_user_add(owner, owner).await?;
    client.cluster_user_login(owner, owner).await?;
    client.db_add(owner, db, DbKind::Memory).await?;

    client
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
    client.db_backup(owner, db).await?;
    client
        .db_exec_mut(
            owner,
            db,
            &[QueryBuilder::remove().ids("root").query().into()],
        )
        .await?;

    client.db_convert(owner, db, DbKind::Mapped).await?;
    let db_list = client.db_list().await?.1;
    assert_eq!(db_list[0].db_type, DbKind::Mapped);

    client.db_restore(owner, db).await?;
    let results = client
        .db_exec(
            owner,
            db,
            &[QueryBuilder::select().ids("root").query().into()],
        )
        .await?
        .1;
    assert_eq!(results[0].result, 1);

    let data_dir = cluster.data_dir(client.address());
    let audit = audit_dir(&data_dir, owner, db);
    let backup_audit = backup_audit_dir(&data_dir, owner, db);
    assert!(Path::new(&audit).exists());
    assert!(Path::new(&backup_audit).exists());
    assert_eq!(audit_entries(&audit)?, 1);
    assert_eq!(audit_entries(&backup_audit)?, 1);

    Ok(())
}

pub(super) const TESTS: &[Test] = tests![
    admin_db_backup_restore,
    admin_db_rollback,
    admin_db_audit_trim,
    admin_db_convert,
    admin_cluster_logout_all,
    db_backup,
    db_rollback,
    db_convert,
];
