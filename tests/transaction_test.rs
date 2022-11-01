use agdb::Db;
use agdb::Query;
use agdb::QueryError;
use agdb::QueryResult;
use agdb::Transaction;

#[test]
fn transaction_is_public_type() {
    let _transaction = Transaction::default();
}

#[test]
fn create_transaction_from_db() {
    let db = Db::default();
    let _transaction: Transaction = db.transaction();
}

#[test]
fn exec_takes_query_returns_query_result() {
    let db = Db::default();
    let query = Query::default();
    let transaction = db.transaction();
    let _result: Result<QueryResult, QueryError> = transaction.exec(query);
}

#[test]
fn create_transaction_from_transaction() {
    let db = Db::default();
    let transaction = db.transaction();
    let _nested_transaction: Transaction = transaction.transaction();
}

#[test]
fn transaction_commit() {
    let db = Db::default();
    let transaction = db.transaction();
    let _result: Result<QueryResult, QueryError> = transaction.commit();
}

#[test]
fn transaction_rollback() {
    let db = Db::default();
    let transaction = db.transaction();
    let _result: Result<QueryResult, QueryError> = transaction.rollback();
}
