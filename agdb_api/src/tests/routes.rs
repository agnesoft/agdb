// Admin DB Operations
pub mod admin_db_add_test;
pub mod admin_db_audit_test;
pub mod admin_db_backup_restore_test;
pub mod admin_db_clear_test;
pub mod admin_db_convert_test;
pub mod admin_db_copy_test;
pub mod admin_db_delete_test;
pub mod admin_db_exec_test;
pub mod admin_db_list_test;
pub mod admin_db_optimize_test;
pub mod admin_db_remove_test;
pub mod admin_db_rename_test;
pub mod admin_db_user_add_test;
pub mod admin_db_user_list_test;
pub mod admin_db_user_remove_test;

// Admin User Operations
pub mod admin_status_test;
pub mod admin_user_add_test;
pub mod admin_user_change_password_test;
pub mod admin_user_delete_test;
pub mod admin_user_list_test;
pub mod admin_user_logout_test;

// User DB Operations
pub mod db_add_test;
pub mod db_audit_test;
pub mod db_backup_restore_test;
pub mod db_clear_test;
pub mod db_convert_test;
pub mod db_copy_test;
pub mod db_delete_test;
pub mod db_exec_test;
pub mod db_list_test;
pub mod db_optimize_test;
pub mod db_remove_test;
pub mod db_rename_test;
pub mod db_user_add_test;
pub mod db_user_list;
pub mod db_user_remove_test;

// User Authentication
pub mod user_change_password_test;
pub mod user_login_test;
pub mod user_logout_all_test;
pub mod user_logout_test;
pub mod user_status;

// Miscellaneous (subset)
pub mod cluster_test;
pub mod misc_routes;
