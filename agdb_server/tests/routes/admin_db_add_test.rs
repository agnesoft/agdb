use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn add() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_add_test::add().await
}

#[tokio::test]
async fn add_same_name_with_previous_backup() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_add_test::add_same_name_with_previous_backup().await
}

#[tokio::test]
async fn db_already_exists() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_add_test::db_already_exists().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_add_test::user_not_found().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_add_test::non_admin().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::admin_db_add_test::no_token().await
}
