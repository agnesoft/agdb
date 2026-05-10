use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn memory_to_mapped() -> Result<(), TestError> {
    agdb_api::tests::routes::db_convert_test::memory_to_mapped().await
}

#[tokio::test]
async fn same_type() -> Result<(), TestError> {
    agdb_api::tests::routes::db_convert_test::same_type().await
}

#[tokio::test]
async fn file_to_memory() -> Result<(), TestError> {
    agdb_api::tests::routes::db_convert_test::file_to_memory().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_convert_test::db_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::db_convert_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_convert_test::no_token().await
}

