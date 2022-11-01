use agdb::Db;
use agdb::Query;
use agdb::QueryError;
use agdb::QueryResult;

#[test]
fn db_is_public_type() {
    let _db = Db::default();
}

#[test]
fn exec_takes_query_returns_query_result() {
    let db = Db::default();
    let query = Query::default();
    let _result: Result<QueryResult, QueryError> = db.exec(query);
}
