use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn backup() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::backup().await
}

#[tokio::test]
async fn backup_overwrite() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::backup_overwrite().await
}

#[tokio::test]
async fn backup_of_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::backup_of_backup().await
}

#[tokio::test]
async fn in_memory() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::in_memory().await
}

#[tokio::test]
async fn restore_no_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::restore_no_backup().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_backup_restore_test::no_token().await
}
