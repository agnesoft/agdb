use agdb::Db;
use agdb::DbKey;
use agdb::DbValue;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryResult;

#[test]
fn public_types() {
    let _db = Db::new("").unwrap();
    let _query_error = QueryError::default();
    let _result = QueryResult::default();
    let _db_value = DbValue::from("");
    let _db_key = DbKey::from("");
}

#[test]
fn exec_takes_query_returns_query_result() {
    let mut db = Db::new("").unwrap();
    let query = QueryBuilder::insert().node().query();
    let _result = db.exec_mut(&query);
}
