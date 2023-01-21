use agdb::QueryBuilder;

#[test]
fn search_from_where_key() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .key("key".into())
        .query();
}

#[test]
fn search_from_where_keys() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .keys(&["key".into(), "key2".into()])
        .query();
}
