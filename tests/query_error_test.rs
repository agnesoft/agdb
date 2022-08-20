extern crate agdb;

#[test]
fn query_error_is_public_type() {
    let _error = agdb::QueryError::default();
}
