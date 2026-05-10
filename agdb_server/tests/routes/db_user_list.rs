use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn list() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_list::list().await
}

#[tokio::test]
async fn non_db_user() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_list::non_db_user().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_list::no_token().await
}
