use agdb::Db;
use agdb::QueryBuilder;
use agdb::QueryError;
use agdb::QueryResult;
use agdb::Transaction;

#[test]
fn exec_takes_query_returns_query_result() {
    let db = Db::default();
    let transaction = db.transaction();
    let query = QueryBuilder::insert().node().query();
    let _result = transaction.exec(query);
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
