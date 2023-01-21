use agdb::QueryBuilder;

#[test]
fn search_from_where_not_beyond_key() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .not_beyond()
        .key("key".into())
        .query();
}
