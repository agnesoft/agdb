use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn remove() -> Result<(), TestError> {
    agdb_api::tests::routes::db_remove_test::remove().await
}

#[tokio::test]
async fn remove_with_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_remove_test::remove_with_backup().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_remove_test::db_not_found().await
}

#[tokio::test]
async fn non_owner() -> Result<(), TestError> {
    agdb_api::tests::routes::db_remove_test::non_owner().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_remove_test::no_token().await
}
