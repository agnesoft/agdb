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
