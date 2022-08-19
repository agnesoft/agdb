extern crate agdb;

#[test]
fn query_result_is_public_type() {
    let _result = agdb::QueryResult::default();
}
