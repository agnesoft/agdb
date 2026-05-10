use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn remove() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::remove().await
}

#[tokio::test]
async fn remove_owner() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::remove_owner().await
}

#[tokio::test]
async fn non_admin() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::non_admin().await
}

#[tokio::test]
async fn remove_self() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::remove_self().await
}

#[tokio::test]
async fn remove_self_owner() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::remove_self_owner().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::db_not_found().await
}

#[tokio::test]
async fn user_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::user_not_found().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_user_remove_test::no_token().await
}

