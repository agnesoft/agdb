use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn copy() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::copy().await
}

#[tokio::test]
async fn copy_from_different_user() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::copy_from_different_user().await
}

#[tokio::test]
async fn copy_to_removed() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::copy_to_removed().await
}

#[tokio::test]
async fn target_exists() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::target_exists().await
}

#[tokio::test]
async fn target_self() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::target_self().await
}

#[tokio::test]
async fn invalid() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::invalid().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::db_not_found().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_copy_test::no_token().await
}

