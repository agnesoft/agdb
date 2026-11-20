mod admin_db_audit_trim_test;
mod admin_db_backup_restore_test;
mod admin_db_clear_test;
mod admin_db_convert_test;
mod admin_db_delete_test;
mod admin_db_remove_test;
mod admin_db_rename_test;
mod admin_set_log_level_test;
mod admin_user_delete_test;
mod admin_user_logout_all;
mod cluster_test;
mod cluster_test_extra;
mod db_audit_migration_test;
mod db_audit_test;
mod db_audit_trim_test;
mod db_backup_restore_test;
mod db_clear_test;
mod db_convert_test;
mod db_delete_test;
mod db_exec_test;
mod db_remove_test;
mod db_rename_test;
mod misc_routes;
#[cfg(feature = "tls")]
mod tls;

pub(super) const TESTS: &[&[crate::runner::Test]] = &[
    admin_db_audit_trim_test::TESTS,
    admin_db_backup_restore_test::TESTS,
    admin_db_clear_test::TESTS,
    admin_db_convert_test::TESTS,
    admin_db_delete_test::TESTS,
    admin_db_remove_test::TESTS,
    admin_db_rename_test::TESTS,
    admin_set_log_level_test::TESTS,
    admin_user_delete_test::TESTS,
    admin_user_logout_all::TESTS,
    cluster_test::TESTS,
    cluster_test_extra::TESTS,
    db_audit_migration_test::TESTS,
    db_audit_test::TESTS,
    db_audit_trim_test::TESTS,
    db_backup_restore_test::TESTS,
    db_clear_test::TESTS,
    db_convert_test::TESTS,
    db_delete_test::TESTS,
    db_exec_test::TESTS,
    db_remove_test::TESTS,
    db_rename_test::TESTS,
    misc_routes::TESTS,
    #[cfg(feature = "tls")]
    tls::TESTS,
];
