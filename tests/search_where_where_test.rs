use agdb::Comparison;
use agdb::QueryBuilder;

#[test]
fn search_from_where_where_key_and_key_end_where_and_distance() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .where_()
        .key("key".into())
        .and()
        .key("key2".into())
        .end_where()
        .or()
        .distance(Comparison::LessThan(2.into()))
        .query();
}

#[test]
fn search_from_where_where_key_or_keys_end_where_and_distance() {
    let _query = QueryBuilder::search()
        .from(1.into())
        .where_()
        .where_()
        .key("key".into())
        .or()
        .keys(&["key2".into()])
        .end_where()
        .and()
        .distance(Comparison::LessThan(2.into()))
        .query();
}
