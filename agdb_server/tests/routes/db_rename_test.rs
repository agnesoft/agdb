use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn rename() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::rename().await
}

#[tokio::test]
async fn rename_with_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::rename_with_backup().await
}

#[tokio::test]
async fn non_owner() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::non_owner().await
}

#[tokio::test]
async fn invalid() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::invalid().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::db_not_found().await
}

#[tokio::test]
async fn target_self() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::target_self().await
}

#[tokio::test]
async fn target_exists() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::target_exists().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_rename_test::no_token().await
}

