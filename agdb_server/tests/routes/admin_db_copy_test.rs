use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn copy() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::copy().await
}

#[tokio::test]
async fn copy_to_different_user() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::copy_to_different_user().await
}

#[tokio::test]
async fn copy_target_exists() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::copy_target_exists().await
}

#[tokio::test]
async fn target_self() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::target_self().await
}

#[tokio::test]
async fn invalid() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::invalid().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::db_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_copy_test::no_token().await
}

