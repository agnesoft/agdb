mod admin_db_add_test;
mod admin_db_audit_test;
mod admin_db_audit_trim_test;
mod admin_db_backup_restore_test;
mod admin_db_clear_test;
mod admin_db_convert_test;
mod admin_db_copy_test;
mod admin_db_delete_test;
mod admin_db_exec_test;
mod admin_db_list_test;
mod admin_db_optimize_test;
mod admin_db_remove_test;
mod admin_db_rename_test;
mod admin_db_user_add_test;
mod admin_db_user_list_test;
mod admin_db_user_remove_test;
mod admin_status_test;
mod admin_user_add_test;
mod admin_user_change_password_test;
mod admin_user_delete_test;
mod admin_user_list_test;
mod admin_user_logout_test;
mod cluster_test;
mod dashboard_test;
mod db_add_test;
mod db_audit_test;
mod db_audit_trim_test;
mod db_backup_restore_test;
mod db_clear_test;
mod db_convert_test;
mod db_copy_test;
mod db_delete_test;
mod db_exec_test;
mod db_list_test;
mod db_optimize_test;
mod db_remove_test;
mod db_rename_test;
mod db_user_add_test;
mod db_user_list;
mod db_user_remove_test;
mod misc_routes;
mod user_change_password_test;
mod user_login_test;
mod user_logout_all_test;
mod user_logout_test;
mod user_status;

pub(super) const TESTS: &[&[crate::runner::Test]] = &[
    admin_db_add_test::TESTS,
    admin_db_audit_test::TESTS,
    admin_db_audit_trim_test::TESTS,
    admin_db_backup_restore_test::TESTS,
    admin_db_clear_test::TESTS,
    admin_db_convert_test::TESTS,
    admin_db_copy_test::TESTS,
    admin_db_delete_test::TESTS,
    admin_db_exec_test::TESTS,
    admin_db_list_test::TESTS,
    admin_db_optimize_test::TESTS,
    admin_db_remove_test::TESTS,
    admin_db_rename_test::TESTS,
    admin_db_user_add_test::TESTS,
    admin_db_user_list_test::TESTS,
    admin_db_user_remove_test::TESTS,
    admin_status_test::TESTS,
    admin_user_add_test::TESTS,
    admin_user_change_password_test::TESTS,
    admin_user_delete_test::TESTS,
    admin_user_list_test::TESTS,
    admin_user_logout_test::TESTS,
    cluster_test::TESTS,
    dashboard_test::TESTS,
    db_add_test::TESTS,
    db_audit_test::TESTS,
    db_audit_trim_test::TESTS,
    db_backup_restore_test::TESTS,
    db_clear_test::TESTS,
    db_convert_test::TESTS,
    db_copy_test::TESTS,
    db_delete_test::TESTS,
    db_exec_test::TESTS,
    db_list_test::TESTS,
    db_optimize_test::TESTS,
    db_remove_test::TESTS,
    db_rename_test::TESTS,
    db_user_add_test::TESTS,
    db_user_list::TESTS,
    db_user_remove_test::TESTS,
    misc_routes::TESTS,
    user_change_password_test::TESTS,
    user_login_test::TESTS,
    user_logout_all_test::TESTS,
    user_logout_test::TESTS,
    user_status::TESTS,
];
