use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn delete() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::delete().await
}

#[tokio::test]
async fn delete_in_memory() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::delete_in_memory().await
}

#[tokio::test]
async fn delete_with_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::delete_with_backup().await
}

#[tokio::test]
async fn delete_in_memory_with_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::delete_in_memory_with_backup().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::db_not_found().await
}

#[tokio::test]
async fn non_owner() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::non_owner().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_delete_test::no_token().await
}

