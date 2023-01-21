use agdb::QueryBuilder;

#[test]
fn search_from_where_not_key() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .not()
        .key("key".into())
        .query();
}
