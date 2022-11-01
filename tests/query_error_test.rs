use agdb::QueryError;

#[test]
fn query_error_is_public_type() {
    let _error = QueryError::default();
}
