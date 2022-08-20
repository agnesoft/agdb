extern crate agdb;

#[test]
fn transaction_is_public_type() {
    let _transaction = agdb::Transaction::default();
}

#[test]
fn create_transaction_from_db() {
    let db = agdb::Db::default();
    let _transaction: agdb::Transaction = db.transaction();
}

#[test]
fn exec_takes_query_returns_query_result() {
    let db = agdb::Db::default();
    let query = agdb::Query::default();
    let transaction = db.transaction();
    let _result: Result<agdb::QueryResult, agdb::QueryError> = transaction.exec(query);
}

#[test]
fn create_transaction_from_transaction() {
    let db = agdb::Db::default();
    let transaction = db.transaction();
    let _nested_transaction: agdb::Transaction = transaction.transaction();
}

#[test]
fn transaction_commit() {
    let db = agdb::Db::default();
    let transaction = db.transaction();
    let _result: Result<agdb::QueryResult, agdb::QueryError> = transaction.commit();
}
