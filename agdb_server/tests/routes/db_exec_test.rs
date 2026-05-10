use agdb_api::test_server::test_error::TestError;

#[tokio::test]
async fn read_write() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::read_write().await
}

#[tokio::test]
async fn read_only() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::read_only().await
}

#[tokio::test]
async fn read_queries() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::read_queries().await
}

#[tokio::test]
async fn write_queries() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::write_queries().await
}

#[tokio::test]
async fn use_result_of_previous_query() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_of_previous_query().await
}

#[tokio::test]
async fn use_result_in_subquery() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_in_subquery().await
}

#[tokio::test]
async fn use_result_in_condition() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_in_condition().await
}

#[tokio::test]
async fn use_result_in_search() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_in_search().await
}

#[tokio::test]
async fn use_result_in_insert_ids() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_in_insert_ids().await
}

#[tokio::test]
async fn reentrant_queries() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::reentrant_queries().await
}

#[tokio::test]
async fn use_result_in_search_bad_query() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_in_search_bad_query().await
}

#[tokio::test]
async fn use_result_in_search_empty_result() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_in_search_empty_result().await
}

#[tokio::test]
async fn use_result_bad_query() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_bad_query().await
}

#[tokio::test]
async fn use_result_out_of_bounds() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::use_result_out_of_bounds().await
}

#[tokio::test]
async fn query_error() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::query_error().await
}

#[tokio::test]
async fn permission_denied() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::permission_denied().await
}

#[tokio::test]
async fn db_not_found() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::db_not_found().await
}

#[tokio::test]
async fn someone_elses_db() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::someone_elses_db().await
}

#[tokio::test]
async fn no_token() -> Result<(), TestError> {
    agdb_api::tests::routes::db_exec_test::no_token().await
}

