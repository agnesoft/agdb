use agdb::Db;
use agdb::DbError;
use agdb::DbKey;
use agdb::DbValue;
use agdb::QueryError;
use agdb::QueryResult;

#[test]
fn public_types() {
    let _db = Db::default();
    let _query_error = QueryError::default();
    let _result = QueryResult::default();
    let _db_error = DbError::from("");
    let _db_value = DbValue::from("");
    let _db_key = DbKey::from("");
}

#[test]
fn exec_takes_query_returns_query_result() {
    let _db = Db::default();
}
