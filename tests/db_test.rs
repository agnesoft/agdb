extern crate agdb;

#[test]
fn db_is_public_type() {
    let _db = agdb::Db::default();
}

#[test]
fn exec_takes_query_returns_query_result() {
    let db = agdb::Db::default();
    let query = agdb::Query::default();
    let _result: agdb::QueryResult = db.exec(query);
}
