use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn clear_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::clear_backup().await
}

#[tokio::test]
async fn clear_audit() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::clear_audit().await
}

#[tokio::test]
async fn clear_db() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::clear_db().await
}

#[tokio::test]
async fn clear_db_memory() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::clear_db_memory().await
}

#[tokio::test]
async fn clear_db_memory_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::clear_db_memory_backup().await
}

#[tokio::test]
async fn clear_all() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::clear_all().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_clear_test::no_token().await
}

